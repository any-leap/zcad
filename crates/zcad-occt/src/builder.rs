//! Shape builders for extrusion, revolution, and sweeping

use crate::error::{OcctError, Result};
use crate::shape::OcctShape;
use zcad_geometry_3d::math::{Point3, Vector3};
use zcad_geometry_3d::topology::Wire;

/// Builder for extruded shapes
pub struct ExtrudeBuilder {
    profile: Wire,
    direction: Vector3,
    distance: f64,
}

impl ExtrudeBuilder {
    /// Create a new extrude builder
    pub fn new(profile: Wire, direction: Vector3, distance: f64) -> Self {
        Self {
            profile,
            direction: direction.normalize(),
            distance,
        }
    }

    /// Build the extruded shape
    pub fn build(self) -> Result<OcctShape> {
        #[cfg(feature = "occt")]
        {
            // Would use BRepPrimAPI_MakePrism
            Err(OcctError::OcctNotAvailable)
        }

        #[cfg(not(feature = "occt"))]
        {
            Err(OcctError::OcctNotAvailable)
        }
    }
}

/// Builder for revolved shapes
pub struct RevolveBuilder {
    profile: Wire,
    axis_point: Point3,
    axis_direction: Vector3,
    angle: f64,
}

impl RevolveBuilder {
    /// Create a new revolve builder
    pub fn new(profile: Wire, axis_point: Point3, axis_direction: Vector3) -> Self {
        Self {
            profile,
            axis_point,
            axis_direction: axis_direction.normalize(),
            angle: std::f64::consts::TAU, // Full revolution by default
        }
    }

    /// Set the revolution angle (radians)
    pub fn angle(mut self, angle: f64) -> Self {
        self.angle = angle;
        self
    }

    /// Set the revolution angle in degrees
    pub fn angle_degrees(mut self, degrees: f64) -> Self {
        self.angle = degrees.to_radians();
        self
    }

    /// Build the revolved shape
    pub fn build(self) -> Result<OcctShape> {
        #[cfg(feature = "occt")]
        {
            // Would use BRepPrimAPI_MakeRevol
            Err(OcctError::OcctNotAvailable)
        }

        #[cfg(not(feature = "occt"))]
        {
            Err(OcctError::OcctNotAvailable)
        }
    }
}

/// Builder for swept shapes (profile along a path)
pub struct SweepBuilder {
    profile: Wire,
    path: Wire,
    with_contact: bool,
    with_correction: bool,
}

impl SweepBuilder {
    /// Create a new sweep builder
    pub fn new(profile: Wire, path: Wire) -> Self {
        Self {
            profile,
            path,
            with_contact: false,
            with_correction: false,
        }
    }

    /// Enable contact mode (profile touches the path)
    pub fn with_contact(mut self) -> Self {
        self.with_contact = true;
        self
    }

    /// Enable Frenet trihedron correction
    pub fn with_correction(mut self) -> Self {
        self.with_correction = true;
        self
    }

    /// Build the swept shape
    pub fn build(self) -> Result<OcctShape> {
        #[cfg(feature = "occt")]
        {
            // Would use BRepOffsetAPI_MakePipe
            Err(OcctError::OcctNotAvailable)
        }

        #[cfg(not(feature = "occt"))]
        {
            Err(OcctError::OcctNotAvailable)
        }
    }
}

/// Convenience functions on OcctShape
impl OcctShape {
    /// Extrude a profile
    pub fn extrude(_profile: Wire, direction: Vector3, distance: f64) -> Result<Self> {
        ExtrudeBuilder::new(Wire::empty(), direction, distance).build()
    }

    /// Revolve a profile around an axis
    pub fn revolve(
        _profile: Wire,
        axis_point: Point3,
        axis_direction: Vector3,
        angle: f64,
    ) -> Result<Self> {
        RevolveBuilder::new(Wire::empty(), axis_point, axis_direction)
            .angle(angle)
            .build()
    }
}
