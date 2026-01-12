//! Workspace types - define domain-specific work environments

use crate::metadata::ModuleCategory;
use serde::{Deserialize, Serialize};

/// Workspace type - determines which modules are loaded by default
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkspaceType {
    /// Basic 2D drafting (no domain modules)
    Drafting2D,
    /// Basic 3D modeling
    Modeling3D,
    /// Mechanical design (MCAD)
    MechanicalDesign,
    /// Steel structure BIM
    SteelStructure,
    /// Concrete structure BIM
    ConcreteStructure,
    /// Architecture BIM
    Architecture,
    /// MEP (Mechanical, Electrical, Plumbing)
    Mep,
    /// Electronic design (PCB)
    ElectronicPcb,
    /// Electronic design (Schematic)
    ElectronicSchematic,
    /// Custom workspace
    Custom,
}

impl WorkspaceType {
    /// Get the display name for this workspace type
    pub fn display_name(&self) -> &'static str {
        match self {
            WorkspaceType::Drafting2D => "2D Drafting",
            WorkspaceType::Modeling3D => "3D Modeling",
            WorkspaceType::MechanicalDesign => "Mechanical Design",
            WorkspaceType::SteelStructure => "Steel Structure",
            WorkspaceType::ConcreteStructure => "Concrete Structure",
            WorkspaceType::Architecture => "Architecture",
            WorkspaceType::Mep => "MEP",
            WorkspaceType::ElectronicPcb => "PCB Design",
            WorkspaceType::ElectronicSchematic => "Schematic Design",
            WorkspaceType::Custom => "Custom",
        }
    }

    /// Get the description for this workspace type
    pub fn description(&self) -> &'static str {
        match self {
            WorkspaceType::Drafting2D => "Basic 2D CAD drafting tools",
            WorkspaceType::Modeling3D => "3D solid modeling without domain specialization",
            WorkspaceType::MechanicalDesign => "Part design, assemblies, and engineering drawings",
            WorkspaceType::SteelStructure => "Steel structure detailing and BIM",
            WorkspaceType::ConcreteStructure => "Reinforced concrete design and rebar detailing",
            WorkspaceType::Architecture => "Architectural design and building modeling",
            WorkspaceType::Mep => "Mechanical, electrical, and plumbing systems",
            WorkspaceType::ElectronicPcb => "PCB layout and design",
            WorkspaceType::ElectronicSchematic => "Electronic schematic capture",
            WorkspaceType::Custom => "Custom workspace with user-selected modules",
        }
    }

    /// Get the primary module category for this workspace
    pub fn primary_category(&self) -> ModuleCategory {
        match self {
            WorkspaceType::Drafting2D | WorkspaceType::Modeling3D => ModuleCategory::Core,
            WorkspaceType::MechanicalDesign => ModuleCategory::Mcad,
            WorkspaceType::SteelStructure
            | WorkspaceType::ConcreteStructure
            | WorkspaceType::Architecture
            | WorkspaceType::Mep => ModuleCategory::Aec,
            WorkspaceType::ElectronicPcb | WorkspaceType::ElectronicSchematic => {
                ModuleCategory::Eda
            }
            WorkspaceType::Custom => ModuleCategory::Core,
        }
    }

    /// Get the default modules to load for this workspace
    pub fn default_modules(&self) -> Vec<&'static str> {
        match self {
            WorkspaceType::Drafting2D => vec!["zcad.core"],
            WorkspaceType::Modeling3D => vec!["zcad.core", "zcad.geometry3d"],
            WorkspaceType::MechanicalDesign => {
                vec!["zcad.core", "zcad.geometry3d", "zcad.mcad"]
            }
            WorkspaceType::SteelStructure => {
                vec!["zcad.core", "zcad.geometry3d", "zcad.bim.core", "zcad.bim.steel"]
            }
            WorkspaceType::ConcreteStructure => {
                vec!["zcad.core", "zcad.geometry3d", "zcad.bim.core", "zcad.bim.concrete"]
            }
            WorkspaceType::Architecture => {
                vec!["zcad.core", "zcad.geometry3d", "zcad.bim.core", "zcad.bim.arch"]
            }
            WorkspaceType::Mep => {
                vec!["zcad.core", "zcad.geometry3d", "zcad.bim.core", "zcad.bim.mep"]
            }
            WorkspaceType::ElectronicPcb => {
                vec!["zcad.core", "zcad.eda.core", "zcad.eda.schematic", "zcad.eda.pcb"]
            }
            WorkspaceType::ElectronicSchematic => {
                vec!["zcad.core", "zcad.eda.core", "zcad.eda.schematic"]
            }
            WorkspaceType::Custom => vec!["zcad.core"],
        }
    }

    /// Get all available workspace types
    pub fn all() -> &'static [WorkspaceType] {
        &[
            WorkspaceType::Drafting2D,
            WorkspaceType::Modeling3D,
            WorkspaceType::MechanicalDesign,
            WorkspaceType::SteelStructure,
            WorkspaceType::ConcreteStructure,
            WorkspaceType::Architecture,
            WorkspaceType::Mep,
            WorkspaceType::ElectronicPcb,
            WorkspaceType::ElectronicSchematic,
            WorkspaceType::Custom,
        ]
    }
}

impl Default for WorkspaceType {
    fn default() -> Self {
        WorkspaceType::Drafting2D
    }
}

/// Workspace configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    /// Workspace name
    pub name: String,
    /// Workspace type
    pub workspace_type: WorkspaceType,
    /// Enabled module IDs
    pub enabled_modules: Vec<String>,
    /// UI layout configuration (JSON)
    pub layout: Option<serde_json::Value>,
    /// Custom settings
    pub settings: serde_json::Value,
}

impl Workspace {
    /// Create a new workspace from a type
    pub fn from_type(workspace_type: WorkspaceType) -> Self {
        Self {
            name: workspace_type.display_name().to_string(),
            workspace_type,
            enabled_modules: workspace_type
                .default_modules()
                .iter()
                .map(|s| s.to_string())
                .collect(),
            layout: None,
            settings: serde_json::Value::Object(serde_json::Map::new()),
        }
    }

    /// Create a custom workspace
    pub fn custom(name: impl Into<String>, modules: Vec<String>) -> Self {
        Self {
            name: name.into(),
            workspace_type: WorkspaceType::Custom,
            enabled_modules: modules,
            layout: None,
            settings: serde_json::Value::Object(serde_json::Map::new()),
        }
    }

    /// Add a module to this workspace
    pub fn add_module(&mut self, module_id: impl Into<String>) {
        let id = module_id.into();
        if !self.enabled_modules.contains(&id) {
            self.enabled_modules.push(id);
        }
    }

    /// Remove a module from this workspace
    pub fn remove_module(&mut self, module_id: &str) -> bool {
        if let Some(pos) = self.enabled_modules.iter().position(|m| m == module_id) {
            self.enabled_modules.remove(pos);
            true
        } else {
            false
        }
    }

    /// Check if a module is enabled in this workspace
    pub fn has_module(&self, module_id: &str) -> bool {
        self.enabled_modules.iter().any(|m| m == module_id)
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Self::from_type(WorkspaceType::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_creation() {
        let ws = Workspace::from_type(WorkspaceType::MechanicalDesign);
        assert_eq!(ws.name, "Mechanical Design");
        assert!(ws.has_module("zcad.mcad"));
    }

    #[test]
    fn test_workspace_modules() {
        let mut ws = Workspace::custom("My Workspace", vec!["zcad.core".to_string()]);
        
        ws.add_module("zcad.bim.steel");
        assert!(ws.has_module("zcad.bim.steel"));
        
        ws.remove_module("zcad.bim.steel");
        assert!(!ws.has_module("zcad.bim.steel"));
    }
}
