//! Element type system for domain-specific entities

use crate::error::{ModuleError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Element type identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ElementTypeId(pub String);

impl ElementTypeId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ElementTypeId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl std::fmt::Display for ElementTypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Element domain - which industry/discipline does this element belong to
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ElementDomain {
    /// Core CAD elements (point, line, circle, etc.)
    Core,
    /// Mechanical design elements
    Mcad,
    /// Architecture elements
    Architecture,
    /// Structural elements (beams, columns, etc.)
    Structural,
    /// MEP elements (ducts, pipes, cables)
    Mep,
    /// Electronic elements (components, traces)
    Electronic,
    /// Custom/third-party
    Custom,
}

/// Property definition for element types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyDefinition {
    /// Property name
    pub name: String,
    /// Property type
    pub property_type: PropertyType,
    /// Default value (JSON)
    pub default_value: Option<serde_json::Value>,
    /// Is this property required?
    pub required: bool,
    /// Human-readable label
    pub label: String,
    /// Description
    pub description: String,
    /// Unit (if applicable)
    pub unit: Option<String>,
}

/// Property types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PropertyType {
    /// Boolean value
    Boolean,
    /// Integer value
    Integer,
    /// Floating point value
    Float,
    /// String value
    String,
    /// Length (with unit)
    Length,
    /// Area (with unit)
    Area,
    /// Volume (with unit)
    Volume,
    /// Angle (with unit)
    Angle,
    /// Reference to another element
    ElementRef,
    /// Reference to a material
    MaterialRef,
    /// Reference to a section/profile
    SectionRef,
    /// Enumeration
    Enum,
    /// List of values
    List,
    /// Nested object
    Object,
}

/// Element type definition
pub trait ElementType: Send + Sync {
    /// Unique type identifier (e.g., "zcad.bim.steel.beam")
    fn type_id(&self) -> &ElementTypeId;

    /// Human-readable name
    fn name(&self) -> &str;

    /// Domain this element belongs to
    fn domain(&self) -> ElementDomain;

    /// Module that provides this element type
    fn module_id(&self) -> &str;

    /// Parent type (for inheritance)
    fn parent_type(&self) -> Option<&ElementTypeId> {
        None
    }

    /// Property definitions
    fn properties(&self) -> &[PropertyDefinition];

    /// IFC type mapping (for BIM elements)
    fn ifc_type(&self) -> Option<&str> {
        None
    }

    /// STEP entity mapping (for MCAD elements)
    fn step_entity(&self) -> Option<&str> {
        None
    }

    /// Description
    fn description(&self) -> &str {
        ""
    }

    /// Icon name for UI
    fn icon(&self) -> Option<&str> {
        None
    }
}

/// Registry entry for element types
struct ElementTypeEntry {
    element_type: Arc<dyn ElementType>,
    enabled: bool,
}

/// Registry for all element types from all modules
pub struct ElementTypeRegistry {
    types: HashMap<ElementTypeId, ElementTypeEntry>,
    by_domain: HashMap<ElementDomain, Vec<ElementTypeId>>,
    by_module: HashMap<String, Vec<ElementTypeId>>,
}

impl ElementTypeRegistry {
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
            by_domain: HashMap::new(),
            by_module: HashMap::new(),
        }
    }

    /// Register an element type
    pub fn register(&mut self, element_type: Arc<dyn ElementType>) -> Result<()> {
        let type_id = element_type.type_id().clone();

        if self.types.contains_key(&type_id) {
            return Err(ModuleError::ElementTypeAlreadyRegistered(type_id.to_string()));
        }

        // Index by domain
        self.by_domain
            .entry(element_type.domain())
            .or_default()
            .push(type_id.clone());

        // Index by module
        self.by_module
            .entry(element_type.module_id().to_string())
            .or_default()
            .push(type_id.clone());

        self.types.insert(
            type_id,
            ElementTypeEntry {
                element_type,
                enabled: true,
            },
        );

        Ok(())
    }

    /// Unregister an element type
    pub fn unregister(&mut self, type_id: &ElementTypeId) -> bool {
        if let Some(entry) = self.types.remove(type_id) {
            // Remove from domain index
            if let Some(types) = self.by_domain.get_mut(&entry.element_type.domain()) {
                types.retain(|t| t != type_id);
            }

            // Remove from module index
            if let Some(types) = self.by_module.get_mut(entry.element_type.module_id()) {
                types.retain(|t| t != type_id);
            }

            true
        } else {
            false
        }
    }

    /// Get an element type by ID
    pub fn get(&self, type_id: &ElementTypeId) -> Option<Arc<dyn ElementType>> {
        self.types
            .get(type_id)
            .filter(|e| e.enabled)
            .map(|e| e.element_type.clone())
    }

    /// List all element types
    pub fn all_types(&self) -> Vec<Arc<dyn ElementType>> {
        self.types
            .values()
            .filter(|e| e.enabled)
            .map(|e| e.element_type.clone())
            .collect()
    }

    /// List element types by domain
    pub fn types_by_domain(&self, domain: ElementDomain) -> Vec<Arc<dyn ElementType>> {
        self.by_domain
            .get(&domain)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// List element types by module
    pub fn types_by_module(&self, module_id: &str) -> Vec<Arc<dyn ElementType>> {
        self.by_module
            .get(module_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Check if a type exists
    pub fn contains(&self, type_id: &ElementTypeId) -> bool {
        self.types.contains_key(type_id)
    }

    /// Get the number of registered types
    pub fn len(&self) -> usize {
        self.types.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }
}

impl Default for ElementTypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestElementType {
        type_id: ElementTypeId,
    }

    impl ElementType for TestElementType {
        fn type_id(&self) -> &ElementTypeId {
            &self.type_id
        }

        fn name(&self) -> &str {
            "Test Element"
        }

        fn domain(&self) -> ElementDomain {
            ElementDomain::Core
        }

        fn module_id(&self) -> &str {
            "zcad.test"
        }

        fn properties(&self) -> &[PropertyDefinition] {
            &[]
        }
    }

    #[test]
    fn test_element_type_registration() {
        let mut registry = ElementTypeRegistry::new();

        let element_type = Arc::new(TestElementType {
            type_id: ElementTypeId::new("zcad.test.element"),
        });

        registry.register(element_type).unwrap();

        assert!(registry.contains(&ElementTypeId::new("zcad.test.element")));
        assert_eq!(registry.len(), 1);
    }
}
