//! Material definitions
//!
//! Materials define physical and visual properties of parts.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

static MATERIAL_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Material ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MaterialId(pub u64);

impl MaterialId {
    pub fn new() -> Self {
        Self(MATERIAL_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for MaterialId {
    fn default() -> Self {
        Self::new()
    }
}

/// Material properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    /// Material ID
    pub id: MaterialId,

    /// Material name
    pub name: String,

    /// Density (kg/m³)
    pub density: f64,

    /// Young's modulus (Pa)
    pub youngs_modulus: f64,

    /// Poisson's ratio
    pub poissons_ratio: f64,

    /// Yield strength (Pa)
    pub yield_strength: f64,

    /// Tensile strength (Pa)
    pub tensile_strength: f64,

    /// Thermal conductivity (W/(m·K))
    pub thermal_conductivity: f64,

    /// Coefficient of thermal expansion (1/K)
    pub thermal_expansion: f64,

    /// Display color (RGBA)
    pub color: [f32; 4],

    /// Roughness (0-1 for PBR rendering)
    pub roughness: f32,

    /// Metallic (0-1 for PBR rendering)
    pub metallic: f32,
}

impl Material {
    /// Create a new material
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: MaterialId::new(),
            name: name.into(),
            density: 7800.0, // Steel default
            youngs_modulus: 200e9,
            poissons_ratio: 0.3,
            yield_strength: 250e6,
            tensile_strength: 400e6,
            thermal_conductivity: 50.0,
            thermal_expansion: 12e-6,
            color: [0.7, 0.7, 0.7, 1.0],
            roughness: 0.5,
            metallic: 0.8,
        }
    }

    /// Set density
    pub fn with_density(mut self, density: f64) -> Self {
        self.density = density;
        self
    }

    /// Set color
    pub fn with_color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.color = [r, g, b, 1.0];
        self
    }

    // ========== Preset Materials ==========

    /// Steel (generic)
    pub fn steel() -> Self {
        Self::new("Steel")
            .with_density(7850.0)
            .with_color(0.6, 0.6, 0.65)
    }

    /// Stainless Steel
    pub fn stainless_steel() -> Self {
        let mut mat = Self::new("Stainless Steel")
            .with_density(8000.0)
            .with_color(0.75, 0.75, 0.78);
        mat.metallic = 0.9;
        mat.roughness = 0.2;
        mat
    }

    /// Aluminum (6061)
    pub fn aluminum() -> Self {
        let mut mat = Self::new("Aluminum 6061");
        mat.density = 2700.0;
        mat.youngs_modulus = 69e9;
        mat.yield_strength = 276e6;
        mat.tensile_strength = 310e6;
        mat.color = [0.8, 0.8, 0.82, 1.0];
        mat.metallic = 0.9;
        mat.roughness = 0.3;
        mat
    }

    /// Copper
    pub fn copper() -> Self {
        let mut mat = Self::new("Copper");
        mat.density = 8960.0;
        mat.youngs_modulus = 117e9;
        mat.thermal_conductivity = 401.0;
        mat.color = [0.72, 0.45, 0.2, 1.0];
        mat.metallic = 1.0;
        mat.roughness = 0.2;
        mat
    }

    /// Brass
    pub fn brass() -> Self {
        let mut mat = Self::new("Brass");
        mat.density = 8500.0;
        mat.youngs_modulus = 100e9;
        mat.color = [0.8, 0.6, 0.2, 1.0];
        mat.metallic = 0.9;
        mat.roughness = 0.3;
        mat
    }

    /// Titanium (Ti-6Al-4V)
    pub fn titanium() -> Self {
        let mut mat = Self::new("Titanium");
        mat.density = 4430.0;
        mat.youngs_modulus = 114e9;
        mat.yield_strength = 880e6;
        mat.tensile_strength = 950e6;
        mat.color = [0.55, 0.55, 0.58, 1.0];
        mat.metallic = 0.8;
        mat.roughness = 0.4;
        mat
    }

    /// ABS Plastic
    pub fn abs_plastic() -> Self {
        let mut mat = Self::new("ABS Plastic");
        mat.density = 1050.0;
        mat.youngs_modulus = 2.3e9;
        mat.yield_strength = 40e6;
        mat.color = [0.2, 0.2, 0.2, 1.0];
        mat.metallic = 0.0;
        mat.roughness = 0.6;
        mat
    }

    /// Nylon
    pub fn nylon() -> Self {
        let mut mat = Self::new("Nylon");
        mat.density = 1150.0;
        mat.youngs_modulus = 2.7e9;
        mat.color = [0.9, 0.9, 0.85, 1.0];
        mat.metallic = 0.0;
        mat.roughness = 0.5;
        mat
    }
}

/// Material library
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MaterialLibrary {
    materials: HashMap<MaterialId, Material>,
    by_name: HashMap<String, MaterialId>,
}

impl MaterialLibrary {
    /// Create a new empty library
    pub fn new() -> Self {
        Self {
            materials: HashMap::new(),
            by_name: HashMap::new(),
        }
    }

    /// Create a library with default materials
    pub fn with_defaults() -> Self {
        let mut lib = Self::new();
        lib.add(Material::steel());
        lib.add(Material::stainless_steel());
        lib.add(Material::aluminum());
        lib.add(Material::copper());
        lib.add(Material::brass());
        lib.add(Material::titanium());
        lib.add(Material::abs_plastic());
        lib.add(Material::nylon());
        lib
    }

    /// Add a material to the library
    pub fn add(&mut self, material: Material) -> MaterialId {
        let id = material.id;
        self.by_name.insert(material.name.clone(), id);
        self.materials.insert(id, material);
        id
    }

    /// Get a material by ID
    pub fn get(&self, id: MaterialId) -> Option<&Material> {
        self.materials.get(&id)
    }

    /// Get a material by name
    pub fn get_by_name(&self, name: &str) -> Option<&Material> {
        self.by_name
            .get(name)
            .and_then(|id| self.materials.get(id))
    }

    /// List all material names
    pub fn names(&self) -> Vec<&str> {
        self.by_name.keys().map(|s| s.as_str()).collect()
    }

    /// Get the number of materials
    pub fn len(&self) -> usize {
        self.materials.len()
    }

    /// Check if the library is empty
    pub fn is_empty(&self) -> bool {
        self.materials.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_creation() {
        let steel = Material::steel();
        assert_eq!(steel.name, "Steel");
        assert!((steel.density - 7850.0).abs() < 0.1);
    }

    #[test]
    fn test_material_library() {
        let lib = MaterialLibrary::with_defaults();
        assert!(lib.get_by_name("Steel").is_some());
        assert!(lib.get_by_name("Aluminum 6061").is_some());
    }
}
