//! ZCAD 3D Geometry and Topology Kernel
//!
//! This crate provides 3D geometry primitives and B-Rep topology structures
//! for solid modeling in ZCAD.
//!
//! # Architecture
//!
//! The geometry kernel is organized into layers:
//! - **Primitives**: Basic 3D shapes (box, cylinder, sphere, cone)
//! - **Topology**: B-Rep structure (vertex, edge, face, shell, solid)
//! - **Operations**: Boolean operations, extrusion, revolution
//! - **Queries**: Intersection, distance, containment tests
//!
//! # Relationship with OpenCASCADE
//!
//! This crate defines the abstract interfaces and pure-Rust types.
//! The `zcad-occt` crate provides the OpenCASCADE implementation
//! for complex operations like boolean and filleting.
//!
//! # Example
//!
//! ```rust,ignore
//! use zcad_geometry_3d::prelude::*;
//!
//! // Create a box
//! let box_solid = Solid::make_box(100.0, 50.0, 30.0);
//!
//! // Create a cylinder
//! let cylinder = Solid::make_cylinder(25.0, 60.0);
//!
//! // Boolean subtraction
//! let result = box_solid.subtract(&cylinder);
//! ```

pub mod error;
pub mod math;
pub mod mesh;
pub mod primitives;
pub mod topology;
pub mod transform;

pub mod prelude {
    //! Convenient re-exports for 3D geometry operations
    pub use crate::error::{GeometryError, Result};
    pub use crate::math::{BoundingBox3, Matrix4, Point3, Vector3};
    pub use crate::mesh::{Mesh, MeshBuilder, Triangle, Vertex as MeshVertex};
    pub use crate::primitives::{Box3D, Cone, Cylinder, Sphere, Torus};
    pub use crate::topology::{
        Edge, Face, Shell, Solid, SolidId, Surface, TopoShape, Vertex, Wire,
    };
    pub use crate::transform::Transform3D;
}
