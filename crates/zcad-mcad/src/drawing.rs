//! Engineering drawings (2D views from 3D models)
//!
//! Generate 2D views, dimensions, and annotations from 3D parts/assemblies.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use zcad_core::math::Point2;

static DRAWING_ID_COUNTER: AtomicU64 = AtomicU64::new(1);
static VIEW_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Drawing ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DrawingId(pub u64);

impl DrawingId {
    pub fn new() -> Self {
        Self(DRAWING_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for DrawingId {
    fn default() -> Self {
        Self::new()
    }
}

/// View ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ViewId(pub u64);

impl ViewId {
    pub fn new() -> Self {
        Self(VIEW_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for ViewId {
    fn default() -> Self {
        Self::new()
    }
}

/// Drawing sheet sizes (ISO A series)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SheetSize {
    A0,
    A1,
    A2,
    A3,
    A4,
    Custom { width: u32, height: u32 },
}

impl SheetSize {
    /// Get sheet dimensions in mm
    pub fn dimensions_mm(&self) -> (u32, u32) {
        match self {
            SheetSize::A0 => (841, 1189),
            SheetSize::A1 => (594, 841),
            SheetSize::A2 => (420, 594),
            SheetSize::A3 => (297, 420),
            SheetSize::A4 => (210, 297),
            SheetSize::Custom { width, height } => (*width, *height),
        }
    }
}

impl Default for SheetSize {
    fn default() -> Self {
        SheetSize::A3
    }
}

/// View projection type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewProjection {
    /// First angle projection (ISO)
    FirstAngle,
    /// Third angle projection (ANSI)
    ThirdAngle,
}

impl Default for ViewProjection {
    fn default() -> Self {
        ViewProjection::ThirdAngle
    }
}

/// View type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViewType {
    /// Front view
    Front,
    /// Back view
    Back,
    /// Top view
    Top,
    /// Bottom view
    Bottom,
    /// Left view
    Left,
    /// Right view
    Right,
    /// Isometric view
    Isometric,
    /// Section view
    Section {
        cutting_plane: SectionPlane,
    },
    /// Detail view
    Detail {
        center: Point2,
        radius: f64,
        scale_factor: f64,
    },
    /// Auxiliary view
    Auxiliary {
        direction: (f64, f64, f64),
    },
}

/// Section plane definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionPlane {
    /// Points defining the cutting line (can be multiple segments)
    pub points: Vec<Point2>,
    /// Offset from plane
    pub offset: f64,
}

/// A drawing view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawingView {
    /// View ID
    pub id: ViewId,

    /// View name (e.g., "FRONT", "SECTION A-A")
    pub name: String,

    /// View type
    pub view_type: ViewType,

    /// Position on sheet (center)
    pub position: Point2,

    /// Scale (e.g., 1.0 = 1:1, 0.5 = 1:2)
    pub scale: f64,

    /// Rotation (degrees)
    pub rotation: f64,

    /// Show hidden lines?
    pub hidden_lines: bool,

    /// Show center lines?
    pub center_lines: bool,
}

impl DrawingView {
    /// Create a new view
    pub fn new(name: impl Into<String>, view_type: ViewType, position: Point2) -> Self {
        Self {
            id: ViewId::new(),
            name: name.into(),
            view_type,
            position,
            scale: 1.0,
            rotation: 0.0,
            hidden_lines: true,
            center_lines: true,
        }
    }

    /// Set the scale
    pub fn with_scale(mut self, scale: f64) -> Self {
        self.scale = scale;
        self
    }
}

/// An engineering drawing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Drawing {
    /// Drawing ID
    pub id: DrawingId,

    /// Drawing name/number
    pub name: String,

    /// Sheet size
    pub sheet_size: SheetSize,

    /// Projection type
    pub projection: ViewProjection,

    /// Views on the drawing
    pub views: Vec<DrawingView>,

    /// Drawing scale (default for all views)
    pub default_scale: f64,
}

impl Drawing {
    /// Create a new drawing
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: DrawingId::new(),
            name: name.into(),
            sheet_size: SheetSize::default(),
            projection: ViewProjection::default(),
            views: Vec::new(),
            default_scale: 1.0,
        }
    }

    /// Set the sheet size
    pub fn with_sheet_size(mut self, size: SheetSize) -> Self {
        self.sheet_size = size;
        self
    }

    /// Add a view
    pub fn add_view(&mut self, view: DrawingView) -> ViewId {
        let id = view.id;
        self.views.push(view);
        id
    }

    /// Get a view by ID
    pub fn get_view(&self, id: ViewId) -> Option<&DrawingView> {
        self.views.iter().find(|v| v.id == id)
    }

    /// Remove a view
    pub fn remove_view(&mut self, id: ViewId) -> bool {
        if let Some(pos) = self.views.iter().position(|v| v.id == id) {
            self.views.remove(pos);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drawing_creation() {
        let mut drawing = Drawing::new("Test Drawing");

        let view = DrawingView::new("FRONT", ViewType::Front, Point2::new(100.0, 150.0));
        drawing.add_view(view);

        assert_eq!(drawing.views.len(), 1);
    }
}
