//! Tessellation (mesh generation) for shapes

use crate::error::{OcctError, Result};
use crate::shape::OcctShape;
use zcad_geometry_3d::mesh::Mesh;

/// Parameters for tessellation
#[derive(Debug, Clone)]
pub struct TessellationParams {
    /// Linear deflection (chord height)
    pub linear_deflection: f64,
    /// Angular deflection (radians)
    pub angular_deflection: f64,
    /// Relative mode (deflection relative to shape size)
    pub relative: bool,
}

impl Default for TessellationParams {
    fn default() -> Self {
        Self {
            linear_deflection: 0.1,
            angular_deflection: 0.5_f64.to_radians(),
            relative: true,
        }
    }
}

impl TessellationParams {
    /// Create high quality tessellation params
    pub fn high_quality() -> Self {
        Self {
            linear_deflection: 0.01,
            angular_deflection: 0.1_f64.to_radians(),
            relative: true,
        }
    }

    /// Create low quality (fast) tessellation params
    pub fn low_quality() -> Self {
        Self {
            linear_deflection: 1.0,
            angular_deflection: 1.0_f64.to_radians(),
            relative: true,
        }
    }

    /// Create with specific linear deflection
    pub fn with_linear_deflection(mut self, deflection: f64) -> Self {
        self.linear_deflection = deflection;
        self
    }

    /// Create with specific angular deflection
    pub fn with_angular_deflection(mut self, deflection: f64) -> Self {
        self.angular_deflection = deflection;
        self
    }
}

/// Tessellator for converting shapes to meshes
pub struct Tessellator {
    params: TessellationParams,
}

impl Tessellator {
    /// Create a new tessellator with default parameters
    pub fn new() -> Self {
        Self {
            params: TessellationParams::default(),
        }
    }

    /// Create a tessellator with specific parameters
    pub fn with_params(params: TessellationParams) -> Self {
        Self { params }
    }

    /// Tessellate a shape to a mesh
    pub fn tessellate(&self, shape: &OcctShape) -> Result<Mesh> {
        #[cfg(feature = "occt")]
        {
            // Would use BRepMesh_IncrementalMesh
            Err(OcctError::OcctNotAvailable)
        }

        #[cfg(not(feature = "occt"))]
        {
            // Return cached mesh if available
            if let Some(mesh) = shape.mesh() {
                Ok(mesh.clone())
            } else {
                Err(OcctError::TessellationFailed(
                    "No cached mesh and OCCT not available".into(),
                ))
            }
        }
    }
}

impl Default for Tessellator {
    fn default() -> Self {
        Self::new()
    }
}

impl OcctShape {
    /// Tessellate this shape with default parameters
    pub fn tessellate(&self) -> Result<Mesh> {
        Tessellator::new().tessellate(self)
    }

    /// Tessellate this shape with specific parameters
    pub fn tessellate_with(&self, params: TessellationParams) -> Result<Mesh> {
        Tessellator::with_params(params).tessellate(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tessellate_box() {
        let shape = OcctShape::make_box(10.0, 5.0, 2.0).unwrap();
        let result = shape.tessellate();

        // Box has a mesh cache, so this should work even without OCCT
        assert!(result.is_ok());
        let mesh = result.unwrap();
        assert!(mesh.vertex_count() > 0);
    }
}
