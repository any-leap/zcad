//! Entity identification and management
//!
//! ZCAD Core Data Model: Entity as Object
//!
//! Design Philosophy:
//! - Geometry is the carrier, data and relations are the core
//! - Each entity is not just a shape, but an object with properties
//! - Supports hierarchy (parent/children)
//! - Open property system (PropertyMap)
//! - Entity relations (Relations)

use crate::geometry::Geometry;
use crate::math::{Point2, Vector2};
use crate::properties::Properties;
use crate::transform::Transform2D;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static ENTITY_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId {
    pub id: u64,
    pub generation: u32,
}

impl EntityId {
    pub fn new() -> Self {
        Self {
            id: ENTITY_COUNTER.fetch_add(1, Ordering::Relaxed),
            generation: 0,
        }
    }

    pub fn from_raw(id: u64, generation: u32) -> Self {
        Self { id, generation }
    }

    pub const NULL: EntityId = EntityId { id: 0, generation: 0 };

    pub fn is_null(&self) -> bool {
        self.id == 0
    }
}

impl Default for EntityId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PropertyValue {
    String(String),
    Number(f64),
    Bool(bool),
    Point(Point2),
    EntityRef(EntityId),
    List(Vec<PropertyValue>),
    Map(HashMap<String, PropertyValue>),
}

impl PropertyValue {
    pub fn as_string(&self) -> Option<&str> {
        match self { PropertyValue::String(s) => Some(s), _ => None }
    }
    pub fn as_number(&self) -> Option<f64> {
        match self { PropertyValue::Number(n) => Some(*n), _ => None }
    }
    pub fn as_bool(&self) -> Option<bool> {
        match self { PropertyValue::Bool(b) => Some(*b), _ => None }
    }
    pub fn as_entity_ref(&self) -> Option<EntityId> {
        match self { PropertyValue::EntityRef(id) => Some(*id), _ => None }
    }
}

impl From<String> for PropertyValue { fn from(s: String) -> Self { PropertyValue::String(s) } }
impl From<&str> for PropertyValue { fn from(s: &str) -> Self { PropertyValue::String(s.to_string()) } }
impl From<f64> for PropertyValue { fn from(n: f64) -> Self { PropertyValue::Number(n) } }
impl From<i32> for PropertyValue { fn from(n: i32) -> Self { PropertyValue::Number(n as f64) } }
impl From<bool> for PropertyValue { fn from(b: bool) -> Self { PropertyValue::Bool(b) } }
impl From<EntityId> for PropertyValue { fn from(id: EntityId) -> Self { PropertyValue::EntityRef(id) } }

pub type PropertyMap = HashMap<String, PropertyValue>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityContent {
    Geometry(Geometry),
    BlockDefinition { name: String, base_point: Point2, description: String },
    BlockReference { block_id: EntityId, insertion_point: Point2, scale: Vector2, rotation: f64, attributes: HashMap<String, String> },
    ExternalReference { file_path: PathBuf, insertion_point: Point2, scale: Vector2, rotation: f64, loaded: bool },
    Group { name: Option<String> },
}

impl EntityContent {
    pub fn type_name(&self) -> &'static str {
        match self {
            EntityContent::Geometry(g) => g.type_name(),
            EntityContent::BlockDefinition { .. } => "BlockDefinition",
            EntityContent::BlockReference { .. } => "BlockReference",
            EntityContent::ExternalReference { .. } => "ExternalReference",
            EntityContent::Group { .. } => "Group",
        }
    }
    pub fn is_geometry(&self) -> bool { matches!(self, EntityContent::Geometry(_)) }
    pub fn is_container(&self) -> bool {
        matches!(self, EntityContent::BlockDefinition { .. } | EntityContent::BlockReference { .. } | EntityContent::ExternalReference { .. } | EntityContent::Group { .. })
    }
    pub fn as_geometry(&self) -> Option<&Geometry> {
        match self { EntityContent::Geometry(g) => Some(g), _ => None }
    }
    pub fn as_geometry_mut(&mut self) -> Option<&mut Geometry> {
        match self { EntityContent::Geometry(g) => Some(g), _ => None }
    }
}

impl From<Geometry> for EntityContent {
    fn from(geometry: Geometry) -> Self { EntityContent::Geometry(geometry) }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub relation_type: String,
    pub target_id: EntityId,
    pub metadata: PropertyMap,
}

impl Relation {
    pub fn new(relation_type: impl Into<String>, target_id: EntityId) -> Self {
        Self { relation_type: relation_type.into(), target_id, metadata: HashMap::new() }
    }
    pub fn with_metadata(mut self, key: impl Into<String>, value: PropertyValue) -> Self {
        self.metadata.insert(key.into(), value); self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: EntityId,
    pub content: EntityContent,
    pub visual_properties: Properties,
    pub layer_id: EntityId,
    pub parent: Option<EntityId>,
    pub children: Vec<EntityId>,
    pub local_transform: Transform2D,
    pub properties: PropertyMap,
    pub tags: HashSet<String>,
    pub relations: Vec<Relation>,
    pub visible: bool,
    pub locked: bool,
    pub selected: bool,
}

impl Entity {
    pub fn new(geometry: Geometry) -> Self {
        Self {
            id: EntityId::new(),
            content: EntityContent::Geometry(geometry),
            visual_properties: Properties::default(),
            layer_id: EntityId::NULL,
            parent: None,
            children: Vec::new(),
            local_transform: Transform2D::identity(),
            properties: HashMap::new(),
            tags: HashSet::new(),
            relations: Vec::new(),
            visible: true,
            locked: false,
            selected: false,
        }
    }

    pub fn from_content(content: EntityContent) -> Self {
        Self {
            id: EntityId::new(),
            content,
            visual_properties: Properties::default(),
            layer_id: EntityId::NULL,
            parent: None,
            children: Vec::new(),
            local_transform: Transform2D::identity(),
            properties: HashMap::new(),
            tags: HashSet::new(),
            relations: Vec::new(),
            visible: true,
            locked: false,
            selected: false,
        }
    }

    pub fn block_definition(name: impl Into<String>, base_point: Point2) -> Self {
        Self::from_content(EntityContent::BlockDefinition { name: name.into(), base_point, description: String::new() })
    }

    pub fn block_reference(block_id: EntityId, insertion_point: Point2, scale: Vector2, rotation: f64) -> Self {
        let mut e = Self::from_content(EntityContent::BlockReference { block_id, insertion_point, scale, rotation, attributes: HashMap::new() });
        e.local_transform = Transform2D::translation(insertion_point.x, insertion_point.y)
            .then(&Transform2D::rotation(rotation)).then(&Transform2D::scale(scale.x, scale.y));
        e
    }

    pub fn group(name: Option<String>) -> Self { Self::from_content(EntityContent::Group { name }) }

    pub fn geometry(&self) -> Option<&Geometry> { self.content.as_geometry() }
    pub fn geometry_mut(&mut self) -> Option<&mut Geometry> { self.content.as_geometry_mut() }

    pub fn bounding_box(&self) -> crate::math::BoundingBox2 {
        match &self.content { EntityContent::Geometry(g) => g.bounding_box(), _ => crate::math::BoundingBox2::empty() }
    }

    pub fn with_layer(mut self, layer_id: EntityId) -> Self { self.layer_id = layer_id; self }
    pub fn with_properties(mut self, props: Properties) -> Self { self.visual_properties = props; self }
    pub fn with_property(mut self, key: impl Into<String>, value: impl Into<PropertyValue>) -> Self {
        self.properties.insert(key.into(), value.into()); self
    }
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self { self.tags.insert(tag.into()); self }
    pub fn with_tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        for t in tags { self.tags.insert(t.into()); } self
    }
    pub fn with_relation(mut self, r: Relation) -> Self { self.relations.push(r); self }
    pub fn with_parent(mut self, parent_id: EntityId) -> Self { self.parent = Some(parent_id); self }
    pub fn with_transform(mut self, t: Transform2D) -> Self { self.local_transform = t; self }

    pub fn get_property(&self, key: &str) -> Option<&PropertyValue> { self.properties.get(key) }
    pub fn set_property(&mut self, key: impl Into<String>, value: impl Into<PropertyValue>) { self.properties.insert(key.into(), value.into()); }
    pub fn remove_property(&mut self, key: &str) -> Option<PropertyValue> { self.properties.remove(key) }
    pub fn has_tag(&self, tag: &str) -> bool { self.tags.contains(tag) }
    pub fn add_tag(&mut self, tag: impl Into<String>) { self.tags.insert(tag.into()); }
    pub fn remove_tag(&mut self, tag: &str) -> bool { self.tags.remove(tag) }
    pub fn add_relation(&mut self, r: Relation) { self.relations.push(r); }
    pub fn relations_of_type<'a>(&'a self, rt: &'a str) -> impl Iterator<Item = &'a Relation> + 'a {
        self.relations.iter().filter(move |r| r.relation_type == rt)
    }
    pub fn related_ids(&self, rt: &str) -> Vec<EntityId> {
        self.relations_of_type(rt).map(|r| r.target_id).collect()
    }
}
