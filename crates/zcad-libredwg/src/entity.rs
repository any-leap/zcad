//! DWG entity types
//!
//! This module provides safe Rust representations of DWG entities.

/// 2D point
#[derive(Debug, Clone, Copy, Default)]
pub struct DwgPoint2 {
    pub x: f64,
    pub y: f64,
}

impl DwgPoint2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// 3D point
#[derive(Debug, Clone, Copy, Default)]
pub struct DwgPoint3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl DwgPoint3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn to_2d(self) -> DwgPoint2 {
        DwgPoint2::new(self.x, self.y)
    }
}

/// Color representation
#[derive(Debug, Clone, Copy)]
pub struct DwgColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub index: Option<u8>,
}

impl Default for DwgColor {
    fn default() -> Self {
        Self {
            r: 255,
            g: 255,
            b: 255,
            index: Some(7), // White by default
        }
    }
}

impl DwgColor {
    /// Create color from RGB values
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, index: None }
    }

    /// Create color from AutoCAD Color Index (ACI)
    pub fn from_aci(index: u8) -> Self {
        let (r, g, b) = aci_to_rgb(index);
        Self { r, g, b, index: Some(index) }
    }
}

/// Entity type enumeration
#[derive(Debug, Clone)]
pub enum DwgEntityType {
    Line {
        start: DwgPoint3,
        end: DwgPoint3,
    },
    Circle {
        center: DwgPoint3,
        radius: f64,
    },
    Arc {
        center: DwgPoint3,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
    },
    Polyline {
        points: Vec<DwgPoint3>,
        closed: bool,
    },
    LwPolyline {
        points: Vec<DwgPoint2>,
        bulges: Vec<f64>,
        closed: bool,
    },
    Point {
        position: DwgPoint3,
    },
    Text {
        position: DwgPoint3,
        text: String,
        height: f64,
        rotation: f64,
    },
    MText {
        position: DwgPoint3,
        text: String,
        height: f64,
        width: f64,
    },
    Ellipse {
        center: DwgPoint3,
        major_axis: DwgPoint3,
        ratio: f64,
        start_angle: f64,
        end_angle: f64,
    },
    Spline {
        control_points: Vec<DwgPoint3>,
        knots: Vec<f64>,
        degree: u32,
        closed: bool,
    },
    Insert {
        block_name: String,
        position: DwgPoint3,
        scale: DwgPoint3,
        rotation: f64,
    },
    Dimension {
        def_point: DwgPoint3,
        text_mid_point: DwgPoint3,
        dimension_type: u8,
    },
    Hatch {
        pattern_name: String,
        solid_fill: bool,
    },
    /// Unknown or unsupported entity type
    Unknown {
        type_name: String,
    },
}

/// A DWG entity with common properties
#[derive(Debug, Clone)]
pub struct DwgEntity {
    /// Entity handle (unique identifier within the file)
    pub handle: u64,
    /// Layer name
    pub layer: String,
    /// Entity color
    pub color: DwgColor,
    /// Line weight in 100ths of mm (-1 = ByLayer, -2 = ByBlock)
    pub lineweight: i16,
    /// Linetype name
    pub linetype: String,
    /// Entity type and geometry
    pub entity_type: DwgEntityType,
}

impl DwgEntity {
    /// Create a new entity with default properties
    pub fn new(entity_type: DwgEntityType) -> Self {
        Self {
            handle: 0,
            layer: "0".to_string(),
            color: DwgColor::default(),
            lineweight: -1, // ByLayer
            linetype: "BYLAYER".to_string(),
            entity_type,
        }
    }
}

/// Convert AutoCAD Color Index (ACI) to RGB
pub fn aci_to_rgb(index: u8) -> (u8, u8, u8) {
    match index {
        0 => (0, 0, 0),       // ByBlock
        1 => (255, 0, 0),     // Red
        2 => (255, 255, 0),   // Yellow
        3 => (0, 255, 0),     // Green
        4 => (0, 255, 255),   // Cyan
        5 => (0, 0, 255),     // Blue
        6 => (255, 0, 255),   // Magenta
        7 => (255, 255, 255), // White/Black
        8 => (128, 128, 128), // Dark gray
        9 => (192, 192, 192), // Light gray
        10 => (255, 0, 0),    // Red
        11 => (255, 127, 127),
        12 => (204, 0, 0),
        13 => (204, 102, 102),
        14 => (153, 0, 0),
        15 => (153, 76, 76),
        16 => (127, 0, 0),
        17 => (127, 63, 63),
        18 => (76, 0, 0),
        19 => (76, 38, 38),
        20 => (255, 63, 0),
        21 => (255, 159, 127),
        22 => (204, 51, 0),
        23 => (204, 127, 102),
        24 => (153, 38, 0),
        25 => (153, 95, 76),
        30 => (255, 127, 0),  // Orange
        40 => (255, 191, 0),
        50 => (255, 255, 0),  // Yellow
        60 => (191, 255, 0),
        70 => (127, 255, 0),
        80 => (63, 255, 0),
        90 => (0, 255, 0),    // Green
        100 => (0, 255, 63),
        110 => (0, 255, 127),
        120 => (0, 255, 191),
        130 => (0, 255, 255), // Cyan
        140 => (0, 191, 255),
        150 => (0, 127, 255),
        160 => (0, 63, 255),
        170 => (0, 0, 255),   // Blue
        180 => (63, 0, 255),
        190 => (127, 0, 255),
        200 => (191, 0, 255),
        210 => (255, 0, 255), // Magenta
        220 => (255, 0, 191),
        230 => (255, 0, 127),
        240 => (255, 0, 63),
        250 => (51, 51, 51),  // Dark grays
        251 => (91, 91, 91),
        252 => (132, 132, 132),
        253 => (173, 173, 173),
        254 => (214, 214, 214),
        255 => (255, 255, 255), // White
        _ => {
            // For other colors, use a simple approximation
            let hue = (index as f64 / 256.0) * 360.0;
            hsv_to_rgb(hue, 1.0, 1.0)
        }
    }
}

/// Convert HSV to RGB
fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (u8, u8, u8) {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}
