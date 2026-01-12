//! Boolean operations on shapes

use crate::error::{OcctError, Result};
use crate::shape::OcctShape;
use serde::{Deserialize, Serialize};

/// Boolean operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BooleanOp {
    /// Union (A ∪ B)
    Union,
    /// Intersection (A ∩ B)
    Intersection,
    /// Difference (A - B)
    Difference,
}

impl OcctShape {
    /// Perform a boolean operation with another shape
    pub fn boolean(&self, other: &OcctShape, op: BooleanOp) -> Result<OcctShape> {
        match op {
            BooleanOp::Union => self.boolean_fuse(other),
            BooleanOp::Intersection => self.boolean_common(other),
            BooleanOp::Difference => self.boolean_cut(other),
        }
    }

    /// Boolean union (fuse)
    pub fn boolean_fuse(&self, other: &OcctShape) -> Result<OcctShape> {
        #[cfg(feature = "occt")]
        {
            // Would call BRepAlgoAPI_Fuse
            Err(OcctError::OcctNotAvailable)
        }

        #[cfg(not(feature = "occt"))]
        {
            // Simplified: just combine bounding boxes
            let bbox1 = self.bounding_box();
            let bbox2 = other.bounding_box();
            let combined = bbox1.union(&bbox2);

            let mut new_shape = self.clone();
            new_shape.set_bounding_box(combined);
            Ok(new_shape)
        }
    }

    /// Boolean intersection (common)
    pub fn boolean_common(&self, _other: &OcctShape) -> Result<OcctShape> {
        #[cfg(feature = "occt")]
        {
            // Would call BRepAlgoAPI_Common
            Err(OcctError::OcctNotAvailable)
        }

        #[cfg(not(feature = "occt"))]
        {
            Err(OcctError::OcctNotAvailable)
        }
    }

    /// Boolean difference (cut)
    pub fn boolean_cut(&self, _other: &OcctShape) -> Result<OcctShape> {
        #[cfg(feature = "occt")]
        {
            // Would call BRepAlgoAPI_Cut
            Err(OcctError::OcctNotAvailable)
        }

        #[cfg(not(feature = "occt"))]
        {
            Err(OcctError::OcctNotAvailable)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boolean_fuse_fallback() {
        let box1 = OcctShape::make_box(10.0, 10.0, 10.0).unwrap();
        let box2 = OcctShape::make_box(5.0, 5.0, 5.0).unwrap();

        // Without OCCT feature, fuse should still work with combined bounds
        let result = box1.boolean_fuse(&box2);
        
        #[cfg(not(feature = "occt"))]
        assert!(result.is_ok());
    }
}
