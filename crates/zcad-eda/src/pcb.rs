//! PCB layout
//!
//! Defines board, layers, tracks, vias, and component placement.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use zcad_core::math::Point2;

use crate::component::{ComponentId, FootprintPad, PadShape, PadType};
use crate::netlist::NetId;

static LAYER_ID_COUNTER: AtomicU64 = AtomicU64::new(1);
static ITEM_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Layer ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LayerId(pub u64);

impl LayerId {
    pub fn new() -> Self {
        Self(LAYER_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for LayerId {
    fn default() -> Self {
        Self::new()
    }
}

/// Item ID (for any PCB element)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemId(pub u64);

impl ItemId {
    pub fn new() -> Self {
        Self(ITEM_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for ItemId {
    fn default() -> Self {
        Self::new()
    }
}

/// Layer type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayerType {
    /// Copper signal layer
    Signal,
    /// Copper power/ground plane
    Plane,
    /// Solder mask
    Mask,
    /// Solder paste
    Paste,
    /// Silkscreen
    Silkscreen,
    /// Fabrication layer (board outline, dimensions)
    Fabrication,
    /// Courtyard (component keepout)
    Courtyard,
    /// User-defined
    User,
}

/// A PCB layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    /// Layer ID
    pub id: LayerId,
    
    /// Layer name (e.g., "F.Cu", "B.Cu", "In1.Cu")
    pub name: String,
    
    /// Layer type
    pub layer_type: LayerType,
    
    /// Layer number (for ordering)
    pub number: u32,
    
    /// Is this a front (top) side layer?
    pub is_front: bool,
    
    /// Copper weight (oz) for copper layers
    pub copper_weight: Option<f64>,
}

impl Layer {
    pub fn new(name: impl Into<String>, layer_type: LayerType, number: u32) -> Self {
        Self {
            id: LayerId::new(),
            name: name.into(),
            layer_type,
            number,
            is_front: true,
            copper_weight: if matches!(layer_type, LayerType::Signal | LayerType::Plane) {
                Some(1.0)
            } else {
                None
            },
        }
    }

    pub fn front_copper() -> Self {
        Self::new("F.Cu", LayerType::Signal, 0)
    }

    pub fn back_copper() -> Self {
        let mut layer = Self::new("B.Cu", LayerType::Signal, 31);
        layer.is_front = false;
        layer
    }

    pub fn inner_copper(number: u32) -> Self {
        Self::new(format!("In{}.Cu", number), LayerType::Signal, number)
    }
}

/// A PCB track (copper trace)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    /// Item ID
    pub id: ItemId,
    
    /// Start point
    pub start: Point2,
    
    /// End point
    pub end: Point2,
    
    /// Track width (mm)
    pub width: f64,
    
    /// Layer
    pub layer: LayerId,
    
    /// Net ID
    pub net_id: Option<NetId>,
}

impl Track {
    pub fn new(start: Point2, end: Point2, width: f64, layer: LayerId) -> Self {
        Self {
            id: ItemId::new(),
            start,
            end,
            width,
            layer,
            net_id: None,
        }
    }

    pub fn with_net(mut self, net_id: NetId) -> Self {
        self.net_id = Some(net_id);
        self
    }

    pub fn length(&self) -> f64 {
        ((self.end.x - self.start.x).powi(2) + (self.end.y - self.start.y).powi(2)).sqrt()
    }
}

/// Via type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViaType {
    /// Through-hole via (all layers)
    Through,
    /// Blind via (surface to inner layer)
    Blind,
    /// Buried via (inner layers only)
    Buried,
    /// Micro via (single layer span)
    Micro,
}

/// A via (layer connection)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Via {
    /// Item ID
    pub id: ItemId,
    
    /// Center position
    pub position: Point2,
    
    /// Via type
    pub via_type: ViaType,
    
    /// Outer diameter (mm)
    pub diameter: f64,
    
    /// Drill diameter (mm)
    pub drill: f64,
    
    /// Start layer
    pub start_layer: LayerId,
    
    /// End layer
    pub end_layer: LayerId,
    
    /// Net ID
    pub net_id: Option<NetId>,
}

impl Via {
    pub fn through(position: Point2, diameter: f64, drill: f64) -> Self {
        Self {
            id: ItemId::new(),
            position,
            via_type: ViaType::Through,
            diameter,
            drill,
            start_layer: LayerId(0),
            end_layer: LayerId(31),
            net_id: None,
        }
    }

    pub fn with_net(mut self, net_id: NetId) -> Self {
        self.net_id = Some(net_id);
        self
    }
}

/// A pad on the PCB (from footprint)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pad {
    /// Item ID
    pub id: ItemId,
    
    /// Pad number
    pub number: String,
    
    /// Position
    pub position: Point2,
    
    /// Pad type
    pub pad_type: PadType,
    
    /// Shape
    pub shape: PadShape,
    
    /// Size (width, height in mm)
    pub size: (f64, f64),
    
    /// Drill size (for through-hole)
    pub drill: Option<f64>,
    
    /// Rotation (degrees)
    pub rotation: f64,
    
    /// Layers
    pub layers: Vec<LayerId>,
    
    /// Net ID
    pub net_id: Option<NetId>,
    
    /// Parent footprint instance
    pub footprint_id: Option<ItemId>,
}

impl Pad {
    pub fn from_footprint_pad(fp_pad: &FootprintPad, position: Point2) -> Self {
        Self {
            id: ItemId::new(),
            number: fp_pad.number.clone(),
            position: Point2::new(position.x + fp_pad.position.x, position.y + fp_pad.position.y),
            pad_type: fp_pad.pad_type,
            shape: fp_pad.shape,
            size: fp_pad.size,
            drill: fp_pad.drill,
            rotation: fp_pad.rotation,
            layers: Vec::new(), // Will be resolved when placed
            net_id: None,
            footprint_id: None,
        }
    }

    pub fn with_net(mut self, net_id: NetId) -> Self {
        self.net_id = Some(net_id);
        self
    }
}

/// A footprint instance on the PCB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FootprintInstance {
    /// Item ID
    pub id: ItemId,
    
    /// Reference to component
    pub component_id: ComponentId,
    
    /// Reference designator
    pub reference: String,
    
    /// Value
    pub value: Option<String>,
    
    /// Position
    pub position: Point2,
    
    /// Rotation (degrees)
    pub rotation: f64,
    
    /// Is on back side?
    pub on_back: bool,
    
    /// Pads
    pub pads: Vec<Pad>,
}

impl FootprintInstance {
    pub fn new(component_id: ComponentId, reference: impl Into<String>) -> Self {
        Self {
            id: ItemId::new(),
            component_id,
            reference: reference.into(),
            value: None,
            position: Point2::origin(),
            rotation: 0.0,
            on_back: false,
            pads: Vec::new(),
        }
    }

    pub fn at(mut self, x: f64, y: f64) -> Self {
        self.position = Point2::new(x, y);
        self
    }

    pub fn rotated(mut self, degrees: f64) -> Self {
        self.rotation = degrees;
        self
    }

    pub fn on_back(mut self) -> Self {
        self.on_back = true;
        self
    }
}

/// Zone fill type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZoneFillType {
    Solid,
    Hatched,
    None,
}

/// A copper zone (pour/fill area)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zone {
    /// Item ID
    pub id: ItemId,
    
    /// Zone outline
    pub outline: Vec<Point2>,
    
    /// Layer
    pub layer: LayerId,
    
    /// Net ID
    pub net_id: Option<NetId>,
    
    /// Fill type
    pub fill_type: ZoneFillType,
    
    /// Clearance
    pub clearance: f64,
    
    /// Minimum width
    pub min_width: f64,
    
    /// Priority (higher = filled first)
    pub priority: u32,
}

impl Zone {
    pub fn new(outline: Vec<Point2>, layer: LayerId) -> Self {
        Self {
            id: ItemId::new(),
            outline,
            layer,
            net_id: None,
            fill_type: ZoneFillType::Solid,
            clearance: 0.3,
            min_width: 0.25,
            priority: 0,
        }
    }

    pub fn with_net(mut self, net_id: NetId) -> Self {
        self.net_id = Some(net_id);
        self
    }

    pub fn ground_plane(outline: Vec<Point2>, layer: LayerId, ground_net: NetId) -> Self {
        Self::new(outline, layer)
            .with_net(ground_net)
    }
}

/// PCB outline type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BoardOutline {
    Rectangle { width: f64, height: f64 },
    Polygon(Vec<Point2>),
}

/// A complete PCB board
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    /// Board name
    pub name: String,
    
    /// Board outline
    pub outline: BoardOutline,
    
    /// Origin point
    pub origin: Point2,
    
    /// Layer stack
    pub layers: Vec<Layer>,
    
    /// Footprint instances
    pub footprints: Vec<FootprintInstance>,
    
    /// Tracks
    pub tracks: Vec<Track>,
    
    /// Vias
    pub vias: Vec<Via>,
    
    /// Zones
    pub zones: Vec<Zone>,
    
    /// Design rules
    pub design_rules: DesignRules,
}

/// Design rules for the board
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignRules {
    /// Minimum track width (mm)
    pub min_track_width: f64,
    
    /// Minimum clearance (mm)
    pub min_clearance: f64,
    
    /// Minimum via diameter (mm)
    pub min_via_diameter: f64,
    
    /// Minimum via drill (mm)
    pub min_via_drill: f64,
    
    /// Minimum annular ring (mm)
    pub min_annular_ring: f64,
    
    /// Minimum hole size (mm)
    pub min_hole: f64,
}

impl Default for DesignRules {
    fn default() -> Self {
        Self {
            min_track_width: 0.2,
            min_clearance: 0.2,
            min_via_diameter: 0.8,
            min_via_drill: 0.4,
            min_annular_ring: 0.15,
            min_hole: 0.3,
        }
    }
}

impl Board {
    pub fn new(name: impl Into<String>, width: f64, height: f64) -> Self {
        let mut board = Self {
            name: name.into(),
            outline: BoardOutline::Rectangle { width, height },
            origin: Point2::origin(),
            layers: Vec::new(),
            footprints: Vec::new(),
            tracks: Vec::new(),
            vias: Vec::new(),
            zones: Vec::new(),
            design_rules: DesignRules::default(),
        };
        
        // Add default 2-layer stack
        board.layers.push(Layer::front_copper());
        board.layers.push(Layer::back_copper());
        
        board
    }

    pub fn four_layer(name: impl Into<String>, width: f64, height: f64) -> Self {
        let mut board = Self::new(name, width, height);
        
        // Insert inner layers
        board.layers.insert(1, Layer::inner_copper(1));
        board.layers.insert(2, Layer::inner_copper(2));
        
        board
    }

    pub fn add_footprint(&mut self, footprint: FootprintInstance) {
        self.footprints.push(footprint);
    }

    pub fn add_track(&mut self, track: Track) {
        self.tracks.push(track);
    }

    pub fn add_via(&mut self, via: Via) {
        self.vias.push(via);
    }

    pub fn add_zone(&mut self, zone: Zone) {
        self.zones.push(zone);
    }

    pub fn layer_count(&self) -> usize {
        self.layers
            .iter()
            .filter(|l| matches!(l.layer_type, LayerType::Signal | LayerType::Plane))
            .count()
    }

    pub fn find_layer(&self, name: &str) -> Option<&Layer> {
        self.layers.iter().find(|l| l.name == name)
    }

    pub fn front_copper_layer(&self) -> Option<LayerId> {
        self.layers
            .iter()
            .find(|l| l.name == "F.Cu")
            .map(|l| l.id)
    }

    pub fn back_copper_layer(&self) -> Option<LayerId> {
        self.layers
            .iter()
            .find(|l| l.name == "B.Cu")
            .map(|l| l.id)
    }

    /// Find footprint by reference
    pub fn find_footprint(&self, reference: &str) -> Option<&FootprintInstance> {
        self.footprints.iter().find(|f| f.reference == reference)
    }

    /// Get board dimensions
    pub fn dimensions(&self) -> (f64, f64) {
        match &self.outline {
            BoardOutline::Rectangle { width, height } => (*width, *height),
            BoardOutline::Polygon(points) => {
                let min_x = points.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
                let max_x = points.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max);
                let min_y = points.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
                let max_y = points.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max);
                (max_x - min_x, max_y - min_y)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_creation() {
        let board = Board::new("Test PCB", 100.0, 80.0);
        assert_eq!(board.layer_count(), 2);
        assert_eq!(board.dimensions(), (100.0, 80.0));
    }

    #[test]
    fn test_four_layer_board() {
        let board = Board::four_layer("4-Layer", 100.0, 80.0);
        assert_eq!(board.layer_count(), 4);
    }

    #[test]
    fn test_track() {
        let layer_id = LayerId::new();
        let track = Track::new(
            Point2::new(0.0, 0.0),
            Point2::new(10.0, 0.0),
            0.25,
            layer_id,
        );
        assert!((track.length() - 10.0).abs() < 0.001);
    }
}
