//! Triangle mesh representation
//!
//! This module provides data structures for tessellated 3D geometry,
//! used for visualization and export.

use crate::math::{BoundingBox3, Point3, Vector3};
use serde::{Deserialize, Serialize};

/// Mesh vertex with position and normal
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Vertex {
    pub position: Point3,
    pub normal: Vector3,
    pub uv: Option<[f32; 2]>,
}

impl Vertex {
    pub fn new(position: Point3, normal: Vector3) -> Self {
        Self {
            position,
            normal,
            uv: None,
        }
    }

    pub fn with_uv(mut self, u: f32, v: f32) -> Self {
        self.uv = Some([u, v]);
        self
    }
}

/// Triangle face (indices into vertex array)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Triangle {
    pub indices: [u32; 3],
}

impl Triangle {
    pub fn new(i0: u32, i1: u32, i2: u32) -> Self {
        Self {
            indices: [i0, i1, i2],
        }
    }
}

/// Triangle mesh
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub triangles: Vec<Triangle>,
}

impl Mesh {
    /// Create an empty mesh
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            triangles: Vec::new(),
        }
    }

    /// Create a mesh with pre-allocated capacity
    pub fn with_capacity(vertex_count: usize, triangle_count: usize) -> Self {
        Self {
            vertices: Vec::with_capacity(vertex_count),
            triangles: Vec::with_capacity(triangle_count),
        }
    }

    /// Add a vertex and return its index
    pub fn add_vertex(&mut self, vertex: Vertex) -> u32 {
        let index = self.vertices.len() as u32;
        self.vertices.push(vertex);
        index
    }

    /// Add a triangle
    pub fn add_triangle(&mut self, i0: u32, i1: u32, i2: u32) {
        self.triangles.push(Triangle::new(i0, i1, i2));
    }

    /// Get the number of vertices
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Get the number of triangles
    pub fn triangle_count(&self) -> usize {
        self.triangles.len()
    }

    /// Calculate the bounding box
    pub fn bounding_box(&self) -> BoundingBox3 {
        BoundingBox3::from_points(self.vertices.iter().map(|v| v.position))
    }

    /// Calculate the center of the mesh
    pub fn center(&self) -> Point3 {
        if self.vertices.is_empty() {
            return Point3::origin();
        }

        let sum: Vector3 = self
            .vertices
            .iter()
            .map(|v| v.position.coords)
            .fold(Vector3::zeros(), |acc, v| acc + v);

        Point3::from(sum / self.vertices.len() as f64)
    }

    /// Flip all normals
    pub fn flip_normals(&mut self) {
        for vertex in &mut self.vertices {
            vertex.normal = -vertex.normal;
        }
        // Also reverse triangle winding
        for tri in &mut self.triangles {
            tri.indices.swap(1, 2);
        }
    }

    /// Merge another mesh into this one
    pub fn merge(&mut self, other: &Mesh) {
        let offset = self.vertices.len() as u32;

        self.vertices.extend(other.vertices.iter().cloned());

        for tri in &other.triangles {
            self.triangles.push(Triangle::new(
                tri.indices[0] + offset,
                tri.indices[1] + offset,
                tri.indices[2] + offset,
            ));
        }
    }

    /// Calculate surface area
    pub fn surface_area(&self) -> f64 {
        let mut area = 0.0;
        for tri in &self.triangles {
            let v0 = &self.vertices[tri.indices[0] as usize];
            let v1 = &self.vertices[tri.indices[1] as usize];
            let v2 = &self.vertices[tri.indices[2] as usize];

            let edge1 = v1.position - v0.position;
            let edge2 = v2.position - v0.position;
            area += edge1.cross(&edge2).norm() / 2.0;
        }
        area
    }

    /// Check if the mesh is valid
    pub fn is_valid(&self) -> bool {
        let vertex_count = self.vertices.len() as u32;
        self.triangles.iter().all(|tri| {
            tri.indices[0] < vertex_count
                && tri.indices[1] < vertex_count
                && tri.indices[2] < vertex_count
        })
    }
}

impl Default for Mesh {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for constructing meshes
pub struct MeshBuilder {
    mesh: Mesh,
}

impl MeshBuilder {
    pub fn new() -> Self {
        Self { mesh: Mesh::new() }
    }

    pub fn with_capacity(vertex_count: usize, triangle_count: usize) -> Self {
        Self {
            mesh: Mesh::with_capacity(vertex_count, triangle_count),
        }
    }

    /// Add a vertex
    pub fn vertex(&mut self, position: Point3, normal: Vector3) -> u32 {
        self.mesh.add_vertex(Vertex::new(position, normal))
    }

    /// Add a triangle
    pub fn triangle(&mut self, i0: u32, i1: u32, i2: u32) -> &mut Self {
        self.mesh.add_triangle(i0, i1, i2);
        self
    }

    /// Add a quad (as two triangles)
    pub fn quad(&mut self, i0: u32, i1: u32, i2: u32, i3: u32) -> &mut Self {
        self.mesh.add_triangle(i0, i1, i2);
        self.mesh.add_triangle(i0, i2, i3);
        self
    }

    /// Build the mesh
    pub fn build(self) -> Mesh {
        self.mesh
    }
}

impl Default for MeshBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a simple box mesh
pub fn make_box_mesh(dx: f64, dy: f64, dz: f64) -> Mesh {
    let mut builder = MeshBuilder::with_capacity(24, 12);

    // Define 8 corners
    let corners = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(dx, 0.0, 0.0),
        Point3::new(dx, dy, 0.0),
        Point3::new(0.0, dy, 0.0),
        Point3::new(0.0, 0.0, dz),
        Point3::new(dx, 0.0, dz),
        Point3::new(dx, dy, dz),
        Point3::new(0.0, dy, dz),
    ];

    // Face normals
    let normals = [
        Vector3::new(0.0, 0.0, -1.0), // Bottom
        Vector3::new(0.0, 0.0, 1.0),  // Top
        Vector3::new(0.0, -1.0, 0.0), // Front
        Vector3::new(0.0, 1.0, 0.0),  // Back
        Vector3::new(-1.0, 0.0, 0.0), // Left
        Vector3::new(1.0, 0.0, 0.0),  // Right
    ];

    // Define faces (corner indices and normal index)
    let faces = [
        ([0, 3, 2, 1], 0), // Bottom
        ([4, 5, 6, 7], 1), // Top
        ([0, 1, 5, 4], 2), // Front
        ([2, 3, 7, 6], 3), // Back
        ([0, 4, 7, 3], 4), // Left
        ([1, 2, 6, 5], 5), // Right
    ];

    for (corner_indices, normal_idx) in &faces {
        let normal = normals[*normal_idx];
        let v0 = builder.vertex(corners[corner_indices[0]], normal);
        let v1 = builder.vertex(corners[corner_indices[1]], normal);
        let v2 = builder.vertex(corners[corner_indices[2]], normal);
        let v3 = builder.vertex(corners[corner_indices[3]], normal);
        builder.quad(v0, v1, v2, v3);
    }

    builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_mesh() {
        let mesh = make_box_mesh(10.0, 5.0, 2.0);

        assert_eq!(mesh.vertex_count(), 24); // 4 vertices per face * 6 faces
        assert_eq!(mesh.triangle_count(), 12); // 2 triangles per face * 6 faces
        assert!(mesh.is_valid());
    }

    #[test]
    fn test_mesh_merge() {
        let mut mesh1 = make_box_mesh(10.0, 5.0, 2.0);
        let mesh2 = make_box_mesh(5.0, 5.0, 5.0);

        let original_count = mesh1.vertex_count();
        mesh1.merge(&mesh2);

        assert_eq!(mesh1.vertex_count(), original_count + mesh2.vertex_count());
        assert!(mesh1.is_valid());
    }
}
