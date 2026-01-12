//! MCAD Module integration with ZCAD module system

use zcad_module::prelude::*;

/// MCAD Module for ZCAD
pub struct McadModule {
    enabled: bool,
}

impl McadModule {
    pub fn new() -> Self {
        Self { enabled: true }
    }
}

impl Default for McadModule {
    fn default() -> Self {
        Self::new()
    }
}

impl ZcadModule for McadModule {
    fn metadata(&self) -> ModuleMetadata {
        ModuleMetadata::new("zcad.mcad", "Mechanical Design (MCAD)", ModuleCategory::Mcad)
            .with_version(Version::new(0, 1, 0))
            .with_description("Mechanical CAD module for part design, assemblies, and engineering drawings")
            .with_author("ZCAD Contributors")
    }

    fn initialize(&mut self, _ctx: &mut ModuleContext) -> Result<()> {
        tracing::info!("Initializing MCAD module...");
        Ok(())
    }

    fn register_commands(&self, registry: &mut CommandRegistry) {
        // Register MCAD-specific commands
        registry.register(std::sync::Arc::new(NewPartCommand)).ok();
        registry.register(std::sync::Arc::new(ExtrudeCommand)).ok();
        registry.register(std::sync::Arc::new(RevolveCommand)).ok();
        registry.register(std::sync::Arc::new(HoleCommand)).ok();
        registry.register(std::sync::Arc::new(FilletCommand)).ok();
        registry.register(std::sync::Arc::new(ChamferCommand)).ok();
    }

    fn register_element_types(&self, registry: &mut ElementTypeRegistry) {
        // Register MCAD element types
        registry.register(std::sync::Arc::new(PartElementType)).ok();
        registry.register(std::sync::Arc::new(AssemblyElementType)).ok();
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

// ========== Commands ==========

struct NewPartCommand;

impl Command for NewPartCommand {
    fn name(&self) -> &str {
        "NEWPART"
    }

    fn aliases(&self) -> &[&str] {
        &["NP"]
    }

    fn description(&self) -> &str {
        "Create a new part"
    }

    fn execute(&self, _ctx: &CommandContext) -> CommandResult {
        // Would create a new part document
        CommandResult::Message("Created new part".into())
    }

    fn module_id(&self) -> &str {
        "zcad.mcad"
    }

    fn category(&self) -> CommandCategory {
        CommandCategory::Mcad
    }
}

struct ExtrudeCommand;

impl Command for ExtrudeCommand {
    fn name(&self) -> &str {
        "EXTRUDE"
    }

    fn aliases(&self) -> &[&str] {
        &["EXT", "E"]
    }

    fn description(&self) -> &str {
        "Extrude a sketch to create a solid"
    }

    fn execute(&self, _ctx: &CommandContext) -> CommandResult {
        CommandResult::NeedsInput("Select sketch to extrude".into())
    }

    fn module_id(&self) -> &str {
        "zcad.mcad"
    }

    fn category(&self) -> CommandCategory {
        CommandCategory::Mcad
    }
}

struct RevolveCommand;

impl Command for RevolveCommand {
    fn name(&self) -> &str {
        "REVOLVE"
    }

    fn aliases(&self) -> &[&str] {
        &["REV"]
    }

    fn description(&self) -> &str {
        "Revolve a sketch around an axis"
    }

    fn execute(&self, _ctx: &CommandContext) -> CommandResult {
        CommandResult::NeedsInput("Select sketch to revolve".into())
    }

    fn module_id(&self) -> &str {
        "zcad.mcad"
    }

    fn category(&self) -> CommandCategory {
        CommandCategory::Mcad
    }
}

struct HoleCommand;

impl Command for HoleCommand {
    fn name(&self) -> &str {
        "HOLE"
    }

    fn description(&self) -> &str {
        "Create a hole in a solid"
    }

    fn execute(&self, _ctx: &CommandContext) -> CommandResult {
        CommandResult::NeedsInput("Select face for hole placement".into())
    }

    fn module_id(&self) -> &str {
        "zcad.mcad"
    }

    fn category(&self) -> CommandCategory {
        CommandCategory::Mcad
    }
}

struct FilletCommand;

impl Command for FilletCommand {
    fn name(&self) -> &str {
        "FILLET3D"
    }

    fn aliases(&self) -> &[&str] {
        &["FIL3D"]
    }

    fn description(&self) -> &str {
        "Fillet edges of a solid"
    }

    fn execute(&self, _ctx: &CommandContext) -> CommandResult {
        CommandResult::NeedsInput("Select edges to fillet".into())
    }

    fn module_id(&self) -> &str {
        "zcad.mcad"
    }

    fn category(&self) -> CommandCategory {
        CommandCategory::Mcad
    }
}

struct ChamferCommand;

impl Command for ChamferCommand {
    fn name(&self) -> &str {
        "CHAMFER3D"
    }

    fn aliases(&self) -> &[&str] {
        &["CHA3D"]
    }

    fn description(&self) -> &str {
        "Chamfer edges of a solid"
    }

    fn execute(&self, _ctx: &CommandContext) -> CommandResult {
        CommandResult::NeedsInput("Select edges to chamfer".into())
    }

    fn module_id(&self) -> &str {
        "zcad.mcad"
    }

    fn category(&self) -> CommandCategory {
        CommandCategory::Mcad
    }
}

// ========== Element Types ==========

struct PartElementType;

impl ElementType for PartElementType {
    fn type_id(&self) -> &ElementTypeId {
        static ID: std::sync::OnceLock<ElementTypeId> = std::sync::OnceLock::new();
        ID.get_or_init(|| ElementTypeId::new("zcad.mcad.part"))
    }

    fn name(&self) -> &str {
        "Part"
    }

    fn domain(&self) -> ElementDomain {
        ElementDomain::Mcad
    }

    fn module_id(&self) -> &str {
        "zcad.mcad"
    }

    fn properties(&self) -> &[PropertyDefinition] {
        &[]
    }

    fn step_entity(&self) -> Option<&str> {
        Some("PRODUCT")
    }
}

struct AssemblyElementType;

impl ElementType for AssemblyElementType {
    fn type_id(&self) -> &ElementTypeId {
        static ID: std::sync::OnceLock<ElementTypeId> = std::sync::OnceLock::new();
        ID.get_or_init(|| ElementTypeId::new("zcad.mcad.assembly"))
    }

    fn name(&self) -> &str {
        "Assembly"
    }

    fn domain(&self) -> ElementDomain {
        ElementDomain::Mcad
    }

    fn module_id(&self) -> &str {
        "zcad.mcad"
    }

    fn properties(&self) -> &[PropertyDefinition] {
        &[]
    }

    fn step_entity(&self) -> Option<&str> {
        Some("PRODUCT_DEFINITION")
    }
}
