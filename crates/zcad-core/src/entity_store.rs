//! EntityStore - Hierarchical entity storage with query support

use crate::entity::{Entity, EntityId, EntityContent};
use crate::geometry::Geometry;
use crate::math::{BoundingBox2, Point2};
use crate::transform::Transform2D;
use std::collections::{HashMap, HashSet};

/// EntityStore - Hierarchical entity storage
pub struct EntityStore {
    entities: HashMap<EntityId, Entity>,
    roots: HashSet<EntityId>,
    block_definitions: HashMap<String, EntityId>,
}

impl Default for EntityStore {
    fn default() -> Self { Self::new() }
}

impl EntityStore {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            roots: HashSet::new(),
            block_definitions: HashMap::new(),
        }
    }

    pub fn add(&mut self, entity: Entity) -> EntityId {
        let id = entity.id;
        if let EntityContent::BlockDefinition { ref name, .. } = entity.content {
            self.block_definitions.insert(name.clone(), id);
        }
        if entity.parent.is_none() {
            self.roots.insert(id);
        } else if let Some(parent_id) = entity.parent {
            if let Some(parent) = self.entities.get_mut(&parent_id) {
                if !parent.children.contains(&id) {
                    parent.children.push(id);
                }
            }
        }
        self.entities.insert(id, entity);
        id
    }

    pub fn remove(&mut self, id: &EntityId) -> Option<Entity> {
        if let Some(entity) = self.entities.remove(id) {
            self.roots.remove(id);
            if let EntityContent::BlockDefinition { ref name, .. } = entity.content {
                self.block_definitions.remove(name);
            }
            if let Some(parent_id) = entity.parent {
                if let Some(parent) = self.entities.get_mut(&parent_id) {
                    parent.children.retain(|c| c != id);
                }
            }
            for child_id in entity.children.clone() {
                self.remove(&child_id);
            }
            Some(entity)
        } else {
            None
        }
    }

    pub fn get(&self, id: &EntityId) -> Option<&Entity> { self.entities.get(id) }
    pub fn get_mut(&mut self, id: &EntityId) -> Option<&mut Entity> { self.entities.get_mut(id) }

    pub fn update(&mut self, id: &EntityId, entity: Entity) {
        if let Some(old_entity) = self.entities.get(id) {
            let old_parent = old_entity.parent;
            let new_parent = entity.parent;
            if old_parent != new_parent {
                if let Some(old_parent_id) = old_parent {
                    if let Some(parent) = self.entities.get_mut(&old_parent_id) {
                        parent.children.retain(|c| c != id);
                    }
                }
                if let Some(new_parent_id) = new_parent {
                    if let Some(parent) = self.entities.get_mut(&new_parent_id) {
                        if !parent.children.contains(id) { parent.children.push(*id); }
                    }
                }
                if new_parent.is_none() { self.roots.insert(*id); } else { self.roots.remove(id); }
            }
        }
        self.entities.insert(*id, entity);
    }

    pub fn set_parent(&mut self, child_id: &EntityId, parent_id: Option<EntityId>) {
        if let Some(child) = self.entities.get_mut(child_id) {
            let old_parent = child.parent;
            child.parent = parent_id;
            if let Some(old_parent_id) = old_parent {
                if let Some(parent) = self.entities.get_mut(&old_parent_id) {
                    parent.children.retain(|c| c != child_id);
                }
            }
            if let Some(new_parent_id) = parent_id {
                if let Some(parent) = self.entities.get_mut(&new_parent_id) {
                    if !parent.children.contains(child_id) { parent.children.push(*child_id); }
                }
                self.roots.remove(child_id);
            } else {
                self.roots.insert(*child_id);
            }
        }
    }

    pub fn roots(&self) -> impl Iterator<Item = &Entity> {
        self.roots.iter().filter_map(|id| self.entities.get(id))
    }
    pub fn len(&self) -> usize { self.entities.len() }
    pub fn is_empty(&self) -> bool { self.entities.is_empty() }
    pub fn iter(&self) -> impl Iterator<Item = &Entity> { self.entities.values() }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Entity> { self.entities.values_mut() }
    pub fn get_block_definition(&self, name: &str) -> Option<&Entity> {
        self.block_definitions.get(name).and_then(|id| self.entities.get(id))
    }
    pub fn get_block_definition_id(&self, name: &str) -> Option<EntityId> {
        self.block_definitions.get(name).copied()
    }

    pub fn get_world_transform(&self, id: &EntityId) -> Transform2D {
        if let Some(entity) = self.entities.get(id) {
            if let Some(parent_id) = entity.parent {
                let parent_transform = self.get_world_transform(&parent_id);
                parent_transform.then(&entity.local_transform)
            } else { entity.local_transform }
        } else { Transform2D::identity() }
    }

    pub fn get_descendants(&self, id: &EntityId) -> Vec<EntityId> {
        let mut result = Vec::new();
        self.collect_descendants(id, &mut result);
        result
    }
    fn collect_descendants(&self, id: &EntityId, result: &mut Vec<EntityId>) {
        if let Some(entity) = self.entities.get(id) {
            for child_id in &entity.children {
                result.push(*child_id);
                self.collect_descendants(child_id, result);
            }
        }
    }

    pub fn get_ancestors(&self, id: &EntityId) -> Vec<EntityId> {
        let mut result = Vec::new();
        let mut current_id = *id;
        while let Some(entity) = self.entities.get(&current_id) {
            if let Some(parent_id) = entity.parent {
                result.push(parent_id);
                current_id = parent_id;
            } else { break; }
        }
        result
    }

    pub fn with_tag(&self, tag: &str) -> Vec<&Entity> {
        self.entities.values().filter(|e| e.has_tag(tag)).collect()
    }
    pub fn of_type(&self, type_name: &str) -> Vec<&Entity> {
        self.entities.values().filter(|e| e.content.type_name() == type_name).collect()
    }
    pub fn with_geometry_type<F>(&self, predicate: F) -> Vec<&Entity>
    where F: Fn(&Geometry) -> bool {
        self.entities.values().filter(|e| e.geometry().map(&predicate).unwrap_or(false)).collect()
    }
    pub fn in_rect(&self, rect: &BoundingBox2) -> Vec<&Entity> {
        self.entities.values().filter(|e| e.bounding_box().intersects(rect)).collect()
    }
    pub fn near_point(&self, point: Point2, tolerance: f64) -> Vec<&Entity> {
        let rect = BoundingBox2::new(
            Point2::new(point.x - tolerance, point.y - tolerance),
            Point2::new(point.x + tolerance, point.y + tolerance),
        );
        self.in_rect(&rect)
    }
    pub fn clear(&mut self) {
        self.entities.clear();
        self.roots.clear();
        self.block_definitions.clear();
    }
}
