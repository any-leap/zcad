//! DWG file reading functionality
//!
//! This module provides a safe wrapper around LibreDWG for reading DWG files.

use std::ffi::{CStr, CString};
use std::path::Path;

use crate::entity::{DwgColor, DwgEntity, DwgEntityType, DwgPoint2, DwgPoint3};
use crate::error::{DwgError, Result};
use crate::sys;

/// A DWG file handle
pub struct DwgFile {
    /// The internal LibreDWG data structure
    dwg: sys::Dwg_Data,
    /// Cached entities
    entities: Vec<DwgEntity>,
    /// Layer names
    layers: Vec<String>,
}

impl DwgFile {
    /// Open a DWG file for reading
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        // Check if file exists
        if !path.exists() {
            return Err(DwgError::FileNotFound(path.display().to_string()));
        }

        // Convert path to C string
        let path_str = path.to_string_lossy();
        let c_path = CString::new(path_str.as_ref())
            .map_err(|_| DwgError::InvalidFile("Invalid path encoding".to_string()))?;

        // Initialize DWG structure
        let mut dwg: sys::Dwg_Data = unsafe { std::mem::zeroed() };

        // Read the DWG file
        let result = unsafe { sys::dwg_read_file(c_path.as_ptr(), &mut dwg) };

        if result != 0 {
            // Free any allocated memory
            unsafe { sys::dwg_free(&mut dwg) };
            return Err(DwgError::from_error_code(result));
        }

        // Create the wrapper
        let mut dwg_file = DwgFile {
            dwg,
            entities: Vec::new(),
            layers: Vec::new(),
        };

        // Parse layers
        dwg_file.parse_layers();

        // Parse entities
        dwg_file.parse_entities();

        Ok(dwg_file)
    }

    /// Get all entities in the DWG file
    pub fn entities(&self) -> &[DwgEntity] {
        &self.entities
    }

    /// Get all layer names
    pub fn layers(&self) -> &[String] {
        &self.layers
    }

    /// Get the DWG version string
    pub fn version(&self) -> String {
        let version = unsafe { self.dwg.header.version };
        format!("DWG version: {:?}", version)
    }

    /// Parse layers from the DWG data
    fn parse_layers(&mut self) {
        unsafe {
            let num_objects = self.dwg.num_objects as usize;
            
            for i in 0..num_objects {
                let obj = self.dwg.object.add(i);
                if obj.is_null() {
                    continue;
                }

                let obj_ref = &*obj;
                
                // Check if this is a LAYER object
                if obj_ref.fixedtype == sys::DWG_OBJECT_TYPE_DWG_TYPE_LAYER {
                    if let Some(name) = self.get_layer_name(obj) {
                        self.layers.push(name);
                    }
                }
            }
        }

        // Ensure we always have the "0" layer
        if !self.layers.contains(&"0".to_string()) {
            self.layers.insert(0, "0".to_string());
        }
    }

    /// Get layer name from object
    unsafe fn get_layer_name(&self, _obj: *const sys::Dwg_Object) -> Option<String> {
        // Layer name extraction is complex due to LibreDWG's structure
        // For now, return None and handle layers by entity reference
        None
    }

    /// Parse all entities from the DWG data
    fn parse_entities(&mut self) {
        unsafe {
            let num_objects = self.dwg.num_objects as usize;

            for i in 0..num_objects {
                let obj = self.dwg.object.add(i);
                if obj.is_null() {
                    continue;
                }

                let obj_ref = &*obj;

                // Only process entity objects (not tables, etc.)
                if obj_ref.supertype != sys::DWG_OBJECT_SUPERTYPE_DWG_SUPERTYPE_ENTITY {
                    continue;
                }

                if let Some(entity) = self.convert_entity(obj) {
                    self.entities.push(entity);
                }
            }
        }
    }

    /// Convert a LibreDWG object to a DwgEntity
    unsafe fn convert_entity(&self, obj: *const sys::Dwg_Object) -> Option<DwgEntity> {
        let obj_ref = &*obj;
        
        // Get common entity properties
        let handle = obj_ref.handle.value;
        let layer = self.get_entity_layer(obj).unwrap_or_else(|| "0".to_string());
        let color = self.get_entity_color(obj);

        // Convert based on entity type
        let entity_type = match obj_ref.fixedtype {
            sys::DWG_OBJECT_TYPE_DWG_TYPE_LINE => self.convert_line(obj)?,
            sys::DWG_OBJECT_TYPE_DWG_TYPE_CIRCLE => self.convert_circle(obj)?,
            sys::DWG_OBJECT_TYPE_DWG_TYPE_ARC => self.convert_arc(obj)?,
            sys::DWG_OBJECT_TYPE_DWG_TYPE_LWPOLYLINE => self.convert_lwpolyline(obj)?,
            sys::DWG_OBJECT_TYPE_DWG_TYPE_POLYLINE_2D => self.convert_polyline(obj)?,
            sys::DWG_OBJECT_TYPE_DWG_TYPE_POLYLINE_3D => self.convert_polyline(obj)?,
            sys::DWG_OBJECT_TYPE_DWG_TYPE_POINT => self.convert_point(obj)?,
            sys::DWG_OBJECT_TYPE_DWG_TYPE_TEXT => self.convert_text(obj)?,
            sys::DWG_OBJECT_TYPE_DWG_TYPE_MTEXT => self.convert_mtext(obj)?,
            sys::DWG_OBJECT_TYPE_DWG_TYPE_ELLIPSE => self.convert_ellipse(obj)?,
            sys::DWG_OBJECT_TYPE_DWG_TYPE_SPLINE => self.convert_spline(obj)?,
            sys::DWG_OBJECT_TYPE_DWG_TYPE_INSERT => self.convert_insert(obj)?,
            _ => {
                // Unknown entity type - skip silently
                return None;
            }
        };

        Some(DwgEntity {
            handle,
            layer,
            color,
            lineweight: -1,
            linetype: "BYLAYER".to_string(),
            entity_type,
        })
    }

    /// Get the layer name for an entity
    unsafe fn get_entity_layer(&self, obj: *const sys::Dwg_Object) -> Option<String> {
        let obj_ref = &*obj;
        
        // Get the entity union
        let tio = obj_ref.tio.entity;
        if tio.is_null() {
            return None;
        }

        let entity = &*tio;
        
        // Get layer reference
        let layer_ref = entity.layer;
        if layer_ref.is_null() {
            return None;
        }

        // Try to get layer object
        let layer_obj = (*layer_ref).obj;
        if layer_obj.is_null() {
            return None;
        }

        // Get layer name from the layer object
        let layer_tio = (*layer_obj).tio.object;
        if layer_tio.is_null() {
            return None;
        }

        // The layer name is typically in the LAYER object
        // This is a simplified extraction
        Some("0".to_string())
    }

    /// Get the color for an entity
    unsafe fn get_entity_color(&self, obj: *const sys::Dwg_Object) -> DwgColor {
        let obj_ref = &*obj;
        
        let tio = obj_ref.tio.entity;
        if tio.is_null() {
            return DwgColor::default();
        }

        let entity = &*tio;
        let color = entity.color;

        // Extract color index
        let index = color.index as u8;
        DwgColor::from_aci(index)
    }

    // Entity conversion methods

    unsafe fn convert_line(&self, obj: *const sys::Dwg_Object) -> Option<DwgEntityType> {
        let line = sys::dwg_object_to_LINE(obj as *mut _);
        if line.is_null() {
            return None;
        }

        let line_ref = &*line;
        
        Some(DwgEntityType::Line {
            start: DwgPoint3::new(line_ref.start.x, line_ref.start.y, line_ref.start.z),
            end: DwgPoint3::new(line_ref.end.x, line_ref.end.y, line_ref.end.z),
        })
    }

    unsafe fn convert_circle(&self, obj: *const sys::Dwg_Object) -> Option<DwgEntityType> {
        let circle = sys::dwg_object_to_CIRCLE(obj as *mut _);
        if circle.is_null() {
            return None;
        }

        let circle_ref = &*circle;

        Some(DwgEntityType::Circle {
            center: DwgPoint3::new(
                circle_ref.center.x,
                circle_ref.center.y,
                circle_ref.center.z,
            ),
            radius: circle_ref.radius,
        })
    }

    unsafe fn convert_arc(&self, obj: *const sys::Dwg_Object) -> Option<DwgEntityType> {
        let arc = sys::dwg_object_to_ARC(obj as *mut _);
        if arc.is_null() {
            return None;
        }

        let arc_ref = &*arc;

        Some(DwgEntityType::Arc {
            center: DwgPoint3::new(arc_ref.center.x, arc_ref.center.y, arc_ref.center.z),
            radius: arc_ref.radius,
            start_angle: arc_ref.start_angle,
            end_angle: arc_ref.end_angle,
        })
    }

    unsafe fn convert_lwpolyline(&self, obj: *const sys::Dwg_Object) -> Option<DwgEntityType> {
        let pline = sys::dwg_object_to_LWPOLYLINE(obj as *mut _);
        if pline.is_null() {
            return None;
        }

        let pline_ref = &*pline;
        let num_points = pline_ref.num_points as usize;

        let mut points = Vec::with_capacity(num_points);
        let mut bulges = Vec::with_capacity(num_points);

        for i in 0..num_points {
            let pt = pline_ref.points.add(i);
            if !pt.is_null() {
                let pt_ref = &*pt;
                points.push(DwgPoint2::new(pt_ref.x, pt_ref.y));
            }

            if !pline_ref.bulges.is_null() {
                let bulge = *pline_ref.bulges.add(i);
                bulges.push(bulge);
            }
        }

        // Pad bulges if needed
        while bulges.len() < points.len() {
            bulges.push(0.0);
        }

        let closed = (pline_ref.flag & 1) != 0;

        Some(DwgEntityType::LwPolyline {
            points,
            bulges,
            closed,
        })
    }

    unsafe fn convert_polyline(&self, obj: *const sys::Dwg_Object) -> Option<DwgEntityType> {
        // POLYLINE_2D and POLYLINE_3D have vertices as separate objects
        // For simplicity, we extract what we can from the header
        let obj_ref = &*obj;
        
        // This is a simplified implementation
        // Full implementation would need to follow VERTEX references
        Some(DwgEntityType::Polyline {
            points: Vec::new(),
            closed: false,
        })
    }

    unsafe fn convert_point(&self, obj: *const sys::Dwg_Object) -> Option<DwgEntityType> {
        let point = sys::dwg_object_to_POINT(obj as *mut _);
        if point.is_null() {
            return None;
        }

        let point_ref = &*point;

        Some(DwgEntityType::Point {
            position: DwgPoint3::new(point_ref.x, point_ref.y, point_ref.z),
        })
    }

    unsafe fn convert_text(&self, obj: *const sys::Dwg_Object) -> Option<DwgEntityType> {
        let text = sys::dwg_object_to_TEXT(obj as *mut _);
        if text.is_null() {
            return None;
        }

        let text_ref = &*text;

        let text_content = if !text_ref.text_value.is_null() {
            CStr::from_ptr(text_ref.text_value)
                .to_string_lossy()
                .into_owned()
        } else {
            String::new()
        };

        Some(DwgEntityType::Text {
            position: DwgPoint3::new(
                text_ref.ins_pt.x,
                text_ref.ins_pt.y,
                0.0,
            ),
            text: text_content,
            height: text_ref.height,
            rotation: text_ref.rotation,
        })
    }

    unsafe fn convert_mtext(&self, obj: *const sys::Dwg_Object) -> Option<DwgEntityType> {
        let mtext = sys::dwg_object_to_MTEXT(obj as *mut _);
        if mtext.is_null() {
            return None;
        }

        let mtext_ref = &*mtext;

        let text_content = if !mtext_ref.text.is_null() {
            CStr::from_ptr(mtext_ref.text)
                .to_string_lossy()
                .into_owned()
        } else {
            String::new()
        };

        Some(DwgEntityType::MText {
            position: DwgPoint3::new(
                mtext_ref.ins_pt.x,
                mtext_ref.ins_pt.y,
                mtext_ref.ins_pt.z,
            ),
            text: text_content,
            height: mtext_ref.text_height,
            width: mtext_ref.rect_width,
        })
    }

    unsafe fn convert_ellipse(&self, obj: *const sys::Dwg_Object) -> Option<DwgEntityType> {
        let ellipse = sys::dwg_object_to_ELLIPSE(obj as *mut _);
        if ellipse.is_null() {
            return None;
        }

        let ellipse_ref = &*ellipse;

        Some(DwgEntityType::Ellipse {
            center: DwgPoint3::new(
                ellipse_ref.center.x,
                ellipse_ref.center.y,
                ellipse_ref.center.z,
            ),
            major_axis: DwgPoint3::new(
                ellipse_ref.sm_axis.x,
                ellipse_ref.sm_axis.y,
                ellipse_ref.sm_axis.z,
            ),
            ratio: ellipse_ref.axis_ratio,
            start_angle: ellipse_ref.start_angle,
            end_angle: ellipse_ref.end_angle,
        })
    }

    unsafe fn convert_spline(&self, obj: *const sys::Dwg_Object) -> Option<DwgEntityType> {
        let spline = sys::dwg_object_to_SPLINE(obj as *mut _);
        if spline.is_null() {
            return None;
        }

        let spline_ref = &*spline;
        let num_ctrl_pts = spline_ref.num_ctrl_pts as usize;
        let num_knots = spline_ref.num_knots as usize;

        let mut control_points = Vec::with_capacity(num_ctrl_pts);
        let mut knots = Vec::with_capacity(num_knots);

        for i in 0..num_ctrl_pts {
            if !spline_ref.ctrl_pts.is_null() {
                let pt = &*spline_ref.ctrl_pts.add(i);
                control_points.push(DwgPoint3::new(pt.x, pt.y, pt.z));
            }
        }

        for i in 0..num_knots {
            if !spline_ref.knots.is_null() {
                knots.push(*spline_ref.knots.add(i));
            }
        }

        let closed = (spline_ref.flag & 1) != 0;

        Some(DwgEntityType::Spline {
            control_points,
            knots,
            degree: spline_ref.degree as u32,
            closed,
        })
    }

    unsafe fn convert_insert(&self, obj: *const sys::Dwg_Object) -> Option<DwgEntityType> {
        let insert = sys::dwg_object_to_INSERT(obj as *mut _);
        if insert.is_null() {
            return None;
        }

        let insert_ref = &*insert;

        // Get block name from block header reference
        let block_name = "BLOCK".to_string(); // Simplified

        Some(DwgEntityType::Insert {
            block_name,
            position: DwgPoint3::new(
                insert_ref.ins_pt.x,
                insert_ref.ins_pt.y,
                insert_ref.ins_pt.z,
            ),
            scale: DwgPoint3::new(
                insert_ref.scale.x,
                insert_ref.scale.y,
                insert_ref.scale.z,
            ),
            rotation: insert_ref.rotation,
        })
    }
}

impl Drop for DwgFile {
    fn drop(&mut self) {
        unsafe {
            sys::dwg_free(&mut self.dwg);
        }
    }
}

// Note: DwgFile is not Send/Sync because LibreDWG may not be thread-safe.
// The sys::Dwg_Data contains raw pointers which make it !Send and !Sync by default.
