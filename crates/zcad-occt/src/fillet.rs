//! Fillet and chamfer operations

use crate::error::Result;
use crate::shape::OcctShape;

/// Builder for fillet operations
pub struct FilletBuilder {
    shape: OcctShape,
    radius: f64,
    edges: Vec<usize>, // Edge indices to fillet
}

impl FilletBuilder {
    /// Create a new fillet builder
    pub fn new(shape: OcctShape, radius: f64) -> Self {
        Self {
            shape,
            radius,
            edges: Vec::new(),
        }
    }

    /// Add an edge to fillet (by index)
    pub fn add_edge(mut self, edge_index: usize) -> Self {
        self.edges.push(edge_index);
        self
    }

    /// Fillet all edges
    pub fn all_edges(mut self) -> Self {
        self.edges.clear(); // Empty means all edges
        self
    }

    /// Build the filleted shape
    pub fn build(self) -> Result<OcctShape> {
        #[cfg(feature = "occt")]
        {
            // Would use BRepFilletAPI_MakeFillet
            Err(OcctError::OcctNotAvailable)
        }

        #[cfg(not(feature = "occt"))]
        {
            // Without OCCT, return the original shape
            tracing::warn!(
                "Fillet operation requires OCCT - returning original shape"
            );
            Ok(self.shape)
        }
    }
}

/// Builder for chamfer operations
pub struct ChamferBuilder {
    shape: OcctShape,
    distance: f64,
    edges: Vec<usize>,
}

impl ChamferBuilder {
    /// Create a new chamfer builder
    pub fn new(shape: OcctShape, distance: f64) -> Self {
        Self {
            shape,
            distance,
            edges: Vec::new(),
        }
    }

    /// Add an edge to chamfer (by index)
    pub fn add_edge(mut self, edge_index: usize) -> Self {
        self.edges.push(edge_index);
        self
    }

    /// Chamfer all edges
    pub fn all_edges(mut self) -> Self {
        self.edges.clear();
        self
    }

    /// Build the chamfered shape
    pub fn build(self) -> Result<OcctShape> {
        #[cfg(feature = "occt")]
        {
            // Would use BRepFilletAPI_MakeChamfer
            Err(OcctError::OcctNotAvailable)
        }

        #[cfg(not(feature = "occt"))]
        {
            tracing::warn!(
                "Chamfer operation requires OCCT - returning original shape"
            );
            Ok(self.shape)
        }
    }
}

impl OcctShape {
    /// Create a fillet builder for this shape
    pub fn fillet(&self, radius: f64) -> FilletBuilder {
        FilletBuilder::new(self.clone(), radius)
    }

    /// Fillet all edges with the given radius
    pub fn fillet_all_edges(&self, radius: f64) -> Result<OcctShape> {
        FilletBuilder::new(self.clone(), radius)
            .all_edges()
            .build()
    }

    /// Create a chamfer builder for this shape
    pub fn chamfer(&self, distance: f64) -> ChamferBuilder {
        ChamferBuilder::new(self.clone(), distance)
    }

    /// Chamfer all edges with the given distance
    pub fn chamfer_all_edges(&self, distance: f64) -> Result<OcctShape> {
        ChamferBuilder::new(self.clone(), distance)
            .all_edges()
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fillet_builder() {
        let shape = OcctShape::make_box(10.0, 10.0, 10.0).unwrap();
        let filleted = shape.fillet(1.0).all_edges().build();

        // Without OCCT, should return original shape
        #[cfg(not(feature = "occt"))]
        assert!(filleted.is_ok());
    }
}
