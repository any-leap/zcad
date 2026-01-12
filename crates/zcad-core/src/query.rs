//! Query Engine - Fluent API for entity queries
//!
//! Provides a builder pattern for querying entities:
//! `
//! let doors = query.with_tag("door").where_prop("width", Op::Gt, 900.0).collect();
//! `

use crate::entity::{Entity, EntityId, PropertyValue};
use crate::entity_store::EntityStore;
use crate::geometry::Geometry;
use crate::math::{BoundingBox2, Point2};
use std::collections::HashMap;

/// Comparison operators for property queries
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Op {
    Eq, Ne, Gt, Ge, Lt, Le, Contains,
}

/// Query engine for entity queries
pub struct QueryEngine<'a> {
    store: &'a EntityStore,
}

impl<'a> QueryEngine<'a> {
    pub fn new(store: &'a EntityStore) -> Self {
        Self { store }
    }

    pub fn all(&self) -> QueryBuilder<'a> {
        QueryBuilder::new(self.store)
    }

    pub fn with_tag(&self, tag: &str) -> QueryBuilder<'a> {
        QueryBuilder::new(self.store).with_tag(tag)
    }

    pub fn where_prop(&self, key: &str, op: Op, value: PropertyValue) -> QueryBuilder<'a> {
        QueryBuilder::new(self.store).where_prop(key, op, value)
    }

    pub fn of_type(&self, type_name: &str) -> QueryBuilder<'a> {
        QueryBuilder::new(self.store).of_type(type_name)
    }

    pub fn in_rect(&self, rect: BoundingBox2) -> QueryBuilder<'a> {
        QueryBuilder::new(self.store).in_rect(rect)
    }

    pub fn related_to(&self, id: EntityId, relation_type: &str) -> QueryBuilder<'a> {
        QueryBuilder::new(self.store).related_to(id, relation_type)
    }
}

/// Filter type for query conditions
enum Filter {
    Tag(String),
    Property { key: String, op: Op, value: PropertyValue },
    Type(String),
    Rect(BoundingBox2),
    RelatedTo { id: EntityId, relation_type: String },
    GeometryMatch(fn(&Geometry) -> bool),
}

/// Query builder for fluent queries
pub struct QueryBuilder<'a> {
    store: &'a EntityStore,
    filters: Vec<Filter>,
}

impl<'a> QueryBuilder<'a> {
    fn new(store: &'a EntityStore) -> Self {
        Self { store, filters: Vec::new() }
    }

    pub fn with_tag(mut self, tag: &str) -> Self {
        self.filters.push(Filter::Tag(tag.to_string()));
        self
    }

    pub fn where_prop(mut self, key: &str, op: Op, value: PropertyValue) -> Self {
        self.filters.push(Filter::Property { key: key.to_string(), op, value });
        self
    }

    pub fn of_type(mut self, type_name: &str) -> Self {
        self.filters.push(Filter::Type(type_name.to_string()));
        self
    }

    pub fn in_rect(mut self, rect: BoundingBox2) -> Self {
        self.filters.push(Filter::Rect(rect));
        self
    }

    pub fn related_to(mut self, id: EntityId, relation_type: &str) -> Self {
        self.filters.push(Filter::RelatedTo { id, relation_type: relation_type.to_string() });
        self
    }

    pub fn with_geometry<F>(mut self, predicate: fn(&Geometry) -> bool) -> Self {
        self.filters.push(Filter::GeometryMatch(predicate));
        self
    }

    fn matches(&self, entity: &Entity) -> bool {
        for filter in &self.filters {
            if !self.matches_filter(entity, filter) {
                return false;
            }
        }
        true
    }

    fn matches_filter(&self, entity: &Entity, filter: &Filter) -> bool {
        match filter {
            Filter::Tag(tag) => entity.has_tag(tag),
            Filter::Type(type_name) => entity.content.type_name() == type_name,
            Filter::Rect(rect) => entity.bounding_box().intersects(rect),
            Filter::RelatedTo { id, relation_type } => {
                entity.relations.iter().any(|r| r.target_id == *id && r.relation_type == *relation_type)
            }
            Filter::GeometryMatch(pred) => {
                entity.geometry().map(pred).unwrap_or(false)
            }
            Filter::Property { key, op, value } => {
                if let Some(prop_val) = entity.get_property(key) {
                    Self::compare_values(prop_val, *op, value)
                } else {
                    false
                }
            }
        }
    }

    fn compare_values(a: &PropertyValue, op: Op, b: &PropertyValue) -> bool {
        match (a, b) {
            (PropertyValue::Number(a), PropertyValue::Number(b)) => {
                match op {
                    Op::Eq => (a - b).abs() < 1e-10,
                    Op::Ne => (a - b).abs() >= 1e-10,
                    Op::Gt => a > b,
                    Op::Ge => a >= b,
                    Op::Lt => a < b,
                    Op::Le => a <= b,
                    Op::Contains => false,
                }
            }
            (PropertyValue::String(a), PropertyValue::String(b)) => {
                match op {
                    Op::Eq => a == b,
                    Op::Ne => a != b,
                    Op::Contains => a.contains(b.as_str()),
                    _ => false,
                }
            }
            (PropertyValue::Bool(a), PropertyValue::Bool(b)) => {
                match op {
                    Op::Eq => a == b,
                    Op::Ne => a != b,
                    _ => false,
                }
            }
            _ => false,
        }
    }

    /// Collect matching entity IDs
    pub fn collect_ids(&self) -> Vec<EntityId> {
        self.store.iter().filter(|e| self.matches(e)).map(|e| e.id).collect()
    }

    /// Collect matching entities
    pub fn collect(&self) -> Vec<&'a Entity> {
        self.store.iter().filter(|e| self.matches(e)).collect()
    }

    /// Count matching entities
    pub fn count(&self) -> usize {
        self.store.iter().filter(|e| self.matches(e)).count()
    }

    /// Sum a numeric property of matching entities
    pub fn sum(&self, prop_key: &str) -> f64 {
        self.store.iter()
            .filter(|e| self.matches(e))
            .filter_map(|e| e.get_property(prop_key))
            .filter_map(|v| v.as_number())
            .sum()
    }

    /// Group matching entities by a property value
    pub fn group_by(&self, prop_key: &str) -> HashMap<String, Vec<EntityId>> {
        let mut groups: HashMap<String, Vec<EntityId>> = HashMap::new();
        for entity in self.store.iter().filter(|e| self.matches(e)) {
            let key = entity.get_property(prop_key)
                .and_then(|v| v.as_string())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            groups.entry(key).or_default().push(entity.id);
        }
        groups
    }

    /// Get first matching entity
    pub fn first(&self) -> Option<&'a Entity> {
        self.store.iter().find(|e| self.matches(e))
    }

    /// Check if any entity matches
    pub fn exists(&self) -> bool {
        self.store.iter().any(|e| self.matches(e))
    }
}
