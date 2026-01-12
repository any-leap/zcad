//! BOM (Bill of Materials) Generator
//!
//! Generates material lists from entity data:
//! `
//! let items = bom.generate(query.with_tag("door"), &["material", "width"]);
//! bom.export_csv(&items, "doors.csv")?;
//! `

use crate::entity::{Entity, EntityId, PropertyValue};
use crate::entity_store::EntityStore;
use crate::query::QueryBuilder;
use std::collections::HashMap;
use std::path::Path;

/// BOM item representing a group of entities
#[derive(Debug, Clone)]
pub struct BomItem {
    pub name: String,
    pub quantity: f64,
    pub unit: String,
    pub properties: HashMap<String, PropertyValue>,
    pub entity_ids: Vec<EntityId>,
}

impl BomItem {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            quantity: 1.0,
            unit: "pcs".to_string(),
            properties: HashMap::new(),
            entity_ids: Vec::new(),
        }
    }

    pub fn with_property(mut self, key: impl Into<String>, value: PropertyValue) -> Self {
        self.properties.insert(key.into(), value);
        self
    }

    pub fn get_property(&self, key: &str) -> Option<&PropertyValue> {
        self.properties.get(key)
    }
}

/// BOM Generator
pub struct BomGenerator<'a> {
    store: &'a EntityStore,
}

impl<'a> BomGenerator<'a> {
    pub fn new(store: &'a EntityStore) -> Self {
        Self { store }
    }

    /// Generate BOM from entities matching the query
    pub fn generate(&self, entities: &[&Entity], group_by: &[&str]) -> Vec<BomItem> {
        let mut groups: HashMap<String, BomItem> = HashMap::new();

        for entity in entities {
            // Create group key from properties
            let key = self.create_group_key(entity, group_by);
            
            let item = groups.entry(key.clone()).or_insert_with(|| {
                let mut bom_item = BomItem::new(&key);
                bom_item.quantity = 0.0;
                // Copy properties used for grouping
                for prop_key in group_by {
                    if let Some(val) = entity.get_property(prop_key) {
                        bom_item.properties.insert(prop_key.to_string(), val.clone());
                    }
                }
                bom_item
            });
            
            item.quantity += 1.0;
            item.entity_ids.push(entity.id);
        }

        groups.into_values().collect()
    }

    fn create_group_key(&self, entity: &Entity, group_by: &[&str]) -> String {
        let parts: Vec<String> = group_by.iter()
            .map(|key| {
                entity.get_property(key)
                    .map(|v| self.property_to_string(v))
                    .unwrap_or_else(|| "unknown".to_string())
            })
            .collect();
        
        if parts.is_empty() {
            entity.content.type_name().to_string()
        } else {
            parts.join(" | ")
        }
    }

    fn property_to_string(&self, value: &PropertyValue) -> String {
        match value {
            PropertyValue::String(s) => s.clone(),
            PropertyValue::Number(n) => format!("{}", n),
            PropertyValue::Bool(b) => format!("{}", b),
            _ => "complex".to_string(),
        }
    }

    /// Export BOM to CSV file
    pub fn export_csv(&self, items: &[BomItem], path: &Path) -> std::io::Result<()> {
        use std::io::Write;
        let mut file = std::fs::File::create(path)?;
        
        // Write header
        writeln!(file, "Name,Quantity,Unit")?;
        
        // Write items
        for item in items {
            writeln!(file, "{},{},{}", item.name, item.quantity, item.unit)?;
        }
        
        Ok(())
    }

    /// Calculate total for a property across all items
    pub fn sum_property(&self, items: &[BomItem], prop_key: &str) -> f64 {
        items.iter()
            .filter_map(|item| item.get_property(prop_key))
            .filter_map(|v| v.as_number())
            .sum()
    }

    /// Calculate weighted total (property * quantity)
    pub fn weighted_sum(&self, items: &[BomItem], prop_key: &str) -> f64 {
        items.iter()
            .filter_map(|item| {
                item.get_property(prop_key)
                    .and_then(|v| v.as_number())
                    .map(|n| n * item.quantity)
            })
            .sum()
    }
}
