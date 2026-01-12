//! Command system for module-provided commands

use crate::error::{ModuleError, Result};
use std::collections::HashMap;
use std::sync::Arc;

/// Command execution context
pub struct CommandContext {
    /// Arguments passed to the command
    pub args: Vec<String>,
    /// Named parameters
    pub params: HashMap<String, String>,
}

impl CommandContext {
    pub fn new() -> Self {
        Self {
            args: Vec::new(),
            params: HashMap::new(),
        }
    }

    pub fn with_args(args: Vec<String>) -> Self {
        Self {
            args,
            params: HashMap::new(),
        }
    }

    pub fn get_arg(&self, index: usize) -> Option<&str> {
        self.args.get(index).map(|s| s.as_str())
    }

    pub fn get_param(&self, name: &str) -> Option<&str> {
        self.params.get(name).map(|s| s.as_str())
    }
}

impl Default for CommandContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Command result
#[derive(Debug)]
pub enum CommandResult {
    /// Command completed successfully
    Success,
    /// Command completed with a message
    Message(String),
    /// Command requires more input
    NeedsInput(String),
    /// Command was cancelled
    Cancelled,
    /// Command failed
    Error(String),
}

/// Trait for executable commands
pub trait Command: Send + Sync {
    /// Command name (e.g., "LINE", "BEAM", "COMPONENT")
    fn name(&self) -> &str;

    /// Command aliases (e.g., "L" for "LINE")
    fn aliases(&self) -> &[&str] {
        &[]
    }

    /// Human-readable description
    fn description(&self) -> &str;

    /// Execute the command
    fn execute(&self, ctx: &CommandContext) -> CommandResult;

    /// Get help text
    fn help(&self) -> &str {
        self.description()
    }

    /// Module that provides this command
    fn module_id(&self) -> &str;

    /// Command category for UI grouping
    fn category(&self) -> CommandCategory {
        CommandCategory::Other
    }
}

/// Command categories for UI organization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandCategory {
    /// Drawing commands (LINE, CIRCLE, etc.)
    Draw,
    /// Modification commands (MOVE, COPY, etc.)
    Modify,
    /// View commands (ZOOM, PAN, etc.)
    View,
    /// File commands (OPEN, SAVE, etc.)
    File,
    /// Settings commands
    Settings,
    /// Domain-specific: MCAD
    Mcad,
    /// Domain-specific: BIM
    Bim,
    /// Domain-specific: EDA
    Eda,
    /// Other/uncategorized
    Other,
}

impl CommandCategory {
    pub fn display_name(&self) -> &'static str {
        match self {
            CommandCategory::Draw => "Draw",
            CommandCategory::Modify => "Modify",
            CommandCategory::View => "View",
            CommandCategory::File => "File",
            CommandCategory::Settings => "Settings",
            CommandCategory::Mcad => "Mechanical",
            CommandCategory::Bim => "BIM",
            CommandCategory::Eda => "Electronic",
            CommandCategory::Other => "Other",
        }
    }
}

/// Command registration entry
struct CommandEntry {
    command: Arc<dyn Command>,
    enabled: bool,
}

/// Registry for all commands from all modules
pub struct CommandRegistry {
    commands: HashMap<String, CommandEntry>,
    aliases: HashMap<String, String>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
            aliases: HashMap::new(),
        }
    }

    /// Register a command
    pub fn register(&mut self, command: Arc<dyn Command>) -> Result<()> {
        let name = command.name().to_uppercase();

        if self.commands.contains_key(&name) {
            return Err(ModuleError::CommandAlreadyRegistered(name));
        }

        // Register aliases
        for alias in command.aliases() {
            let alias_upper = alias.to_uppercase();
            if !self.aliases.contains_key(&alias_upper) {
                self.aliases.insert(alias_upper, name.clone());
            }
        }

        self.commands.insert(
            name,
            CommandEntry {
                command,
                enabled: true,
            },
        );

        Ok(())
    }

    /// Unregister a command
    pub fn unregister(&mut self, name: &str) -> bool {
        let name_upper = name.to_uppercase();
        
        if let Some(entry) = self.commands.remove(&name_upper) {
            // Remove aliases
            for alias in entry.command.aliases() {
                self.aliases.remove(&alias.to_uppercase());
            }
            true
        } else {
            false
        }
    }

    /// Get a command by name or alias
    pub fn get(&self, name: &str) -> Option<Arc<dyn Command>> {
        let name_upper = name.to_uppercase();

        // Try direct lookup
        if let Some(entry) = self.commands.get(&name_upper) {
            if entry.enabled {
                return Some(entry.command.clone());
            }
        }

        // Try alias lookup
        if let Some(actual_name) = self.aliases.get(&name_upper) {
            if let Some(entry) = self.commands.get(actual_name) {
                if entry.enabled {
                    return Some(entry.command.clone());
                }
            }
        }

        None
    }

    /// Execute a command by name
    pub fn execute(&self, name: &str, ctx: &CommandContext) -> Option<CommandResult> {
        self.get(name).map(|cmd| cmd.execute(ctx))
    }

    /// List all command names
    pub fn command_names(&self) -> Vec<&str> {
        self.commands.keys().map(|s| s.as_str()).collect()
    }

    /// List commands by category
    pub fn commands_by_category(&self, category: CommandCategory) -> Vec<Arc<dyn Command>> {
        self.commands
            .values()
            .filter(|e| e.enabled && e.command.category() == category)
            .map(|e| e.command.clone())
            .collect()
    }

    /// List commands from a specific module
    pub fn commands_by_module(&self, module_id: &str) -> Vec<Arc<dyn Command>> {
        self.commands
            .values()
            .filter(|e| e.command.module_id() == module_id)
            .map(|e| e.command.clone())
            .collect()
    }

    /// Enable/disable a command
    pub fn set_enabled(&mut self, name: &str, enabled: bool) -> bool {
        let name_upper = name.to_uppercase();
        if let Some(entry) = self.commands.get_mut(&name_upper) {
            entry.enabled = enabled;
            true
        } else {
            false
        }
    }

    /// Check if a command exists
    pub fn contains(&self, name: &str) -> bool {
        let name_upper = name.to_uppercase();
        self.commands.contains_key(&name_upper) || self.aliases.contains_key(&name_upper)
    }

    /// Get the number of registered commands
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestCommand;

    impl Command for TestCommand {
        fn name(&self) -> &str {
            "TEST"
        }

        fn aliases(&self) -> &[&str] {
            &["T", "TST"]
        }

        fn description(&self) -> &str {
            "A test command"
        }

        fn execute(&self, _ctx: &CommandContext) -> CommandResult {
            CommandResult::Success
        }

        fn module_id(&self) -> &str {
            "zcad.test"
        }
    }

    #[test]
    fn test_command_registration() {
        let mut registry = CommandRegistry::new();
        registry.register(Arc::new(TestCommand)).unwrap();

        assert!(registry.contains("TEST"));
        assert!(registry.contains("test"));
        assert!(registry.contains("T"));
        assert!(registry.contains("TST"));
    }

    #[test]
    fn test_command_execution() {
        let mut registry = CommandRegistry::new();
        registry.register(Arc::new(TestCommand)).unwrap();

        let ctx = CommandContext::new();
        let result = registry.execute("T", &ctx);
        assert!(matches!(result, Some(CommandResult::Success)));
    }
}
