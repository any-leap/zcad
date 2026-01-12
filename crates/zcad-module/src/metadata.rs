//! Module metadata definitions

use serde::{Deserialize, Serialize};
use std::fmt;

/// Semantic version
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }

    /// Check if this version is compatible with a required version
    /// Following semver: major must match, minor must be >= required
    pub fn is_compatible_with(&self, required: &Version) -> bool {
        if self.major != required.major {
            return false;
        }
        if self.major == 0 {
            // Pre-1.0: minor version must match exactly
            self.minor == required.minor && self.patch >= required.patch
        } else {
            // Post-1.0: minor version must be >= required
            self.minor > required.minor
                || (self.minor == required.minor && self.patch >= required.patch)
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl Default for Version {
    fn default() -> Self {
        Self::new(0, 1, 0)
    }
}

/// Module category - determines the domain of the module
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModuleCategory {
    /// Core functionality (built-in)
    Core,
    /// Mechanical CAD (manufacturing, product design)
    Mcad,
    /// Architecture, Engineering, Construction / BIM
    Aec,
    /// Electronic Design Automation
    Eda,
    /// Geographic Information System
    Gis,
    /// Specialized domain (ship, aerospace, etc.)
    Specialized,
    /// Third-party extension
    ThirdParty,
}

impl ModuleCategory {
    pub fn display_name(&self) -> &'static str {
        match self {
            ModuleCategory::Core => "Core",
            ModuleCategory::Mcad => "Mechanical Design",
            ModuleCategory::Aec => "AEC/BIM",
            ModuleCategory::Eda => "Electronic Design",
            ModuleCategory::Gis => "GIS",
            ModuleCategory::Specialized => "Specialized",
            ModuleCategory::ThirdParty => "Third Party",
        }
    }
}

impl fmt::Display for ModuleCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Module dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleDependency {
    /// Module ID (e.g., "zcad.bim.core")
    pub module_id: String,
    /// Minimum required version
    pub min_version: Version,
    /// Is this dependency optional?
    pub optional: bool,
}

impl ModuleDependency {
    pub fn required(module_id: impl Into<String>, min_version: Version) -> Self {
        Self {
            module_id: module_id.into(),
            min_version,
            optional: false,
        }
    }

    pub fn optional(module_id: impl Into<String>, min_version: Version) -> Self {
        Self {
            module_id: module_id.into(),
            min_version,
            optional: true,
        }
    }
}

/// Module metadata - describes a module's identity and requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleMetadata {
    /// Unique module identifier (e.g., "zcad.mcad", "zcad.bim.steel")
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Module version
    pub version: Version,
    /// Module category
    pub category: ModuleCategory,
    /// Required dependencies
    pub dependencies: Vec<ModuleDependency>,
    /// Module description
    pub description: String,
    /// Author(s)
    pub authors: Vec<String>,
    /// License
    pub license: Option<String>,
    /// Homepage/repository URL
    pub homepage: Option<String>,
}

impl ModuleMetadata {
    /// Create new module metadata with minimal required fields
    pub fn new(id: impl Into<String>, name: impl Into<String>, category: ModuleCategory) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            version: Version::default(),
            category,
            dependencies: Vec::new(),
            description: String::new(),
            authors: Vec::new(),
            license: None,
            homepage: None,
        }
    }

    /// Builder: set version
    pub fn with_version(mut self, version: Version) -> Self {
        self.version = version;
        self
    }

    /// Builder: add dependency
    pub fn with_dependency(mut self, dep: ModuleDependency) -> Self {
        self.dependencies.push(dep);
        self
    }

    /// Builder: set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Builder: add author
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.authors.push(author.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_compatibility() {
        let v1_0_0 = Version::new(1, 0, 0);
        let v1_0_1 = Version::new(1, 0, 1);
        let v1_1_0 = Version::new(1, 1, 0);
        let v2_0_0 = Version::new(2, 0, 0);

        assert!(v1_0_0.is_compatible_with(&v1_0_0));
        assert!(v1_0_1.is_compatible_with(&v1_0_0));
        assert!(v1_1_0.is_compatible_with(&v1_0_0));
        assert!(!v2_0_0.is_compatible_with(&v1_0_0));
    }

    #[test]
    fn test_pre_1_0_compatibility() {
        let v0_1_0 = Version::new(0, 1, 0);
        let v0_1_1 = Version::new(0, 1, 1);
        let v0_2_0 = Version::new(0, 2, 0);

        assert!(v0_1_0.is_compatible_with(&v0_1_0));
        assert!(v0_1_1.is_compatible_with(&v0_1_0));
        assert!(!v0_2_0.is_compatible_with(&v0_1_0)); // Minor change is breaking pre-1.0
    }
}
