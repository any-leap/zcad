//! CAD file export (STEP, IGES, STL)

use crate::error::{OcctError, Result};
use crate::shape::OcctShape;
use std::path::Path;
use zcad_geometry_3d::mesh::Mesh;

/// Export format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// STEP (ISO 10303)
    Step,
    /// IGES (Initial Graphics Exchange Specification)
    Iges,
    /// STL (Stereolithography)
    Stl,
    /// OBJ (Wavefront)
    Obj,
}

impl ExportFormat {
    /// Get the file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Step => "step",
            ExportFormat::Iges => "iges",
            ExportFormat::Stl => "stl",
            ExportFormat::Obj => "obj",
        }
    }

    /// Detect format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "step" | "stp" => Some(ExportFormat::Step),
            "iges" | "igs" => Some(ExportFormat::Iges),
            "stl" => Some(ExportFormat::Stl),
            "obj" => Some(ExportFormat::Obj),
            _ => None,
        }
    }
}

impl OcctShape {
    /// Export to a file
    pub fn export<P: AsRef<Path>>(&self, path: P, format: ExportFormat) -> Result<()> {
        let path = path.as_ref();

        match format {
            ExportFormat::Step => self.export_step(path),
            ExportFormat::Iges => self.export_iges(path),
            ExportFormat::Stl => self.export_stl(path),
            ExportFormat::Obj => self.export_obj(path),
        }
    }

    /// Export to STEP format
    fn export_step(&self, _path: &Path) -> Result<()> {
        #[cfg(feature = "occt")]
        {
            // Would use STEPControl_Writer
            Err(OcctError::OcctNotAvailable)
        }

        #[cfg(not(feature = "occt"))]
        {
            Err(OcctError::ExportFailed(
                "STEP export requires OCCT".into(),
            ))
        }
    }

    /// Export to IGES format
    fn export_iges(&self, _path: &Path) -> Result<()> {
        #[cfg(feature = "occt")]
        {
            // Would use IGESControl_Writer
            Err(OcctError::OcctNotAvailable)
        }

        #[cfg(not(feature = "occt"))]
        {
            Err(OcctError::ExportFailed(
                "IGES export requires OCCT".into(),
            ))
        }
    }

    /// Export to STL format
    fn export_stl(&self, path: &Path) -> Result<()> {
        // STL doesn't require OCCT - we can export from mesh
        let mesh = self.tessellate()?;
        write_stl(&mesh, path)
    }

    /// Export to OBJ format
    fn export_obj(&self, path: &Path) -> Result<()> {
        let mesh = self.tessellate()?;
        write_obj(&mesh, path)
    }
}

/// Write a mesh to STL format
fn write_stl(mesh: &Mesh, path: &Path) -> Result<()> {
    use std::io::Write;

    let mut file = std::fs::File::create(path)?;

    writeln!(file, "solid mesh")?;

    for tri in &mesh.triangles {
        let v0 = &mesh.vertices[tri.indices[0] as usize];
        let v1 = &mesh.vertices[tri.indices[1] as usize];
        let v2 = &mesh.vertices[tri.indices[2] as usize];

        // Calculate face normal
        let edge1 = v1.position - v0.position;
        let edge2 = v2.position - v0.position;
        let normal = edge1.cross(&edge2).normalize();

        writeln!(file, "  facet normal {} {} {}", normal.x, normal.y, normal.z)?;
        writeln!(file, "    outer loop")?;
        writeln!(
            file,
            "      vertex {} {} {}",
            v0.position.x, v0.position.y, v0.position.z
        )?;
        writeln!(
            file,
            "      vertex {} {} {}",
            v1.position.x, v1.position.y, v1.position.z
        )?;
        writeln!(
            file,
            "      vertex {} {} {}",
            v2.position.x, v2.position.y, v2.position.z
        )?;
        writeln!(file, "    endloop")?;
        writeln!(file, "  endfacet")?;
    }

    writeln!(file, "endsolid mesh")?;

    Ok(())
}

/// Write a mesh to OBJ format
fn write_obj(mesh: &Mesh, path: &Path) -> Result<()> {
    use std::io::Write;

    let mut file = std::fs::File::create(path)?;

    writeln!(file, "# ZCAD OBJ Export")?;
    writeln!(file, "# Vertices: {}", mesh.vertices.len())?;
    writeln!(file, "# Faces: {}", mesh.triangles.len())?;

    // Write vertices
    for v in &mesh.vertices {
        writeln!(file, "v {} {} {}", v.position.x, v.position.y, v.position.z)?;
    }

    // Write normals
    for v in &mesh.vertices {
        writeln!(file, "vn {} {} {}", v.normal.x, v.normal.y, v.normal.z)?;
    }

    // Write faces (OBJ indices are 1-based)
    for tri in &mesh.triangles {
        let i0 = tri.indices[0] + 1;
        let i1 = tri.indices[1] + 1;
        let i2 = tri.indices[2] + 1;
        writeln!(file, "f {}//{} {}//{} {}//{}", i0, i0, i1, i1, i2, i2)?;
    }

    Ok(())
}

/// Import a shape from a file
pub fn import<P: AsRef<Path>>(_path: P) -> Result<OcctShape> {
    #[cfg(feature = "occt")]
    {
        // Would use STEPControl_Reader or IGESControl_Reader
        Err(OcctError::OcctNotAvailable)
    }

    #[cfg(not(feature = "occt"))]
    {
        Err(OcctError::ImportFailed(
            "Import requires OCCT".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_detection() {
        assert_eq!(ExportFormat::from_extension("step"), Some(ExportFormat::Step));
        assert_eq!(ExportFormat::from_extension("STP"), Some(ExportFormat::Step));
        assert_eq!(ExportFormat::from_extension("stl"), Some(ExportFormat::Stl));
        assert_eq!(ExportFormat::from_extension("unknown"), None);
    }
}
