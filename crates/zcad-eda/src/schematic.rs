//! Schematic capture
//!
//! Defines schematic sheets, symbols, wires, and connectivity.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use zcad_core::math::Point2;

use crate::component::ComponentId;
use crate::netlist::NetId;

static SHEET_ID_COUNTER: AtomicU64 = AtomicU64::new(1);
static SYMBOL_INSTANCE_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Sheet ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SheetId(pub u64);

impl SheetId {
    pub fn new() -> Self {
        Self(SHEET_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for SheetId {
    fn default() -> Self {
        Self::new()
    }
}

/// Symbol instance ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SymbolInstanceId(pub u64);

impl SymbolInstanceId {
    pub fn new() -> Self {
        Self(SYMBOL_INSTANCE_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for SymbolInstanceId {
    fn default() -> Self {
        Self::new()
    }
}

/// A symbol instance on a schematic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchematicSymbol {
    /// Instance ID
    pub id: SymbolInstanceId,
    
    /// Reference to component
    pub component_id: ComponentId,
    
    /// Reference designator (e.g., "R1", "C3", "U2")
    pub reference: String,
    
    /// Component value (e.g., "10k", "100nF")
    pub value: Option<String>,
    
    /// Position on sheet
    pub position: Point2,
    
    /// Rotation (degrees: 0, 90, 180, 270)
    pub rotation: f64,
    
    /// Mirror (horizontal flip)
    pub mirror: bool,
    
    /// Unit number (for multi-unit components)
    pub unit: u32,
    
    /// Pin net assignments (pin number -> net ID)
    pub pin_nets: HashMap<String, NetId>,
    
    /// Custom properties
    pub properties: HashMap<String, String>,
}

impl SchematicSymbol {
    pub fn new(component_id: ComponentId, reference: impl Into<String>) -> Self {
        Self {
            id: SymbolInstanceId::new(),
            component_id,
            reference: reference.into(),
            value: None,
            position: Point2::origin(),
            rotation: 0.0,
            mirror: false,
            unit: 1,
            pin_nets: HashMap::new(),
            properties: HashMap::new(),
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

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn assign_pin_net(&mut self, pin_number: impl Into<String>, net_id: NetId) {
        self.pin_nets.insert(pin_number.into(), net_id);
    }
}

/// A wire segment on the schematic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wire {
    /// Start point
    pub start: Point2,
    
    /// End point
    pub end: Point2,
    
    /// Associated net
    pub net_id: Option<NetId>,
}

impl Wire {
    pub fn new(start: Point2, end: Point2) -> Self {
        Self {
            start,
            end,
            net_id: None,
        }
    }

    pub fn from_coords(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        Self::new(Point2::new(x1, y1), Point2::new(x2, y2))
    }

    pub fn with_net(mut self, net_id: NetId) -> Self {
        self.net_id = Some(net_id);
        self
    }

    pub fn length(&self) -> f64 {
        ((self.end.x - self.start.x).powi(2) + (self.end.y - self.start.y).powi(2)).sqrt()
    }

    pub fn is_horizontal(&self) -> bool {
        (self.start.y - self.end.y).abs() < 0.001
    }

    pub fn is_vertical(&self) -> bool {
        (self.start.x - self.end.x).abs() < 0.001
    }
}

/// A bus on the schematic (bundle of nets)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bus {
    /// Bus name (e.g., "DATA[0..7]")
    pub name: String,
    
    /// Start point
    pub start: Point2,
    
    /// End point
    pub end: Point2,
    
    /// Net IDs in this bus
    pub nets: Vec<NetId>,
}

/// Junction point where wires connect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Junction {
    pub position: Point2,
    pub net_id: Option<NetId>,
}

/// A net label (names a net at a point)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetLabel {
    /// Label text
    pub name: String,
    
    /// Position
    pub position: Point2,
    
    /// Rotation
    pub rotation: f64,
    
    /// Associated net
    pub net_id: Option<NetId>,
}

impl NetLabel {
    pub fn new(name: impl Into<String>, x: f64, y: f64) -> Self {
        Self {
            name: name.into(),
            position: Point2::new(x, y),
            rotation: 0.0,
            net_id: None,
        }
    }
}

/// Power symbol (VCC, GND, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerSymbol {
    /// Power net name (e.g., "VCC", "GND", "+5V")
    pub name: String,
    
    /// Position
    pub position: Point2,
    
    /// Rotation
    pub rotation: f64,
    
    /// Associated net
    pub net_id: Option<NetId>,
}

impl PowerSymbol {
    pub fn vcc(x: f64, y: f64) -> Self {
        Self {
            name: "VCC".to_string(),
            position: Point2::new(x, y),
            rotation: 0.0,
            net_id: None,
        }
    }

    pub fn gnd(x: f64, y: f64) -> Self {
        Self {
            name: "GND".to_string(),
            position: Point2::new(x, y),
            rotation: 0.0,
            net_id: None,
        }
    }
}

/// A no-connect marker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoConnect {
    pub position: Point2,
}

/// Text annotation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextAnnotation {
    pub text: String,
    pub position: Point2,
    pub size: f64,
    pub rotation: f64,
}

/// A single schematic sheet/page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchematicPage {
    /// Sheet ID
    pub id: SheetId,
    
    /// Sheet name
    pub name: String,
    
    /// Sheet number
    pub number: u32,
    
    /// Paper size (width, height in mm)
    pub size: (f64, f64),
    
    /// Symbol instances
    pub symbols: Vec<SchematicSymbol>,
    
    /// Wires
    pub wires: Vec<Wire>,
    
    /// Junctions
    pub junctions: Vec<Junction>,
    
    /// Net labels
    pub labels: Vec<NetLabel>,
    
    /// Power symbols
    pub power_symbols: Vec<PowerSymbol>,
    
    /// No-connect markers
    pub no_connects: Vec<NoConnect>,
    
    /// Text annotations
    pub annotations: Vec<TextAnnotation>,
    
    /// Buses
    pub buses: Vec<Bus>,
}

impl SchematicPage {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: SheetId::new(),
            name: name.into(),
            number: 1,
            size: (297.0, 210.0), // A4 landscape
            symbols: Vec::new(),
            wires: Vec::new(),
            junctions: Vec::new(),
            labels: Vec::new(),
            power_symbols: Vec::new(),
            no_connects: Vec::new(),
            annotations: Vec::new(),
            buses: Vec::new(),
        }
    }

    pub fn add_symbol(&mut self, symbol: SchematicSymbol) {
        self.symbols.push(symbol);
    }

    pub fn add_wire(&mut self, wire: Wire) {
        self.wires.push(wire);
    }

    pub fn add_junction(&mut self, x: f64, y: f64) {
        self.junctions.push(Junction {
            position: Point2::new(x, y),
            net_id: None,
        });
    }

    pub fn add_label(&mut self, label: NetLabel) {
        self.labels.push(label);
    }

    pub fn add_power(&mut self, power: PowerSymbol) {
        self.power_symbols.push(power);
    }

    /// Find symbol by reference designator
    pub fn find_symbol(&self, reference: &str) -> Option<&SchematicSymbol> {
        self.symbols.iter().find(|s| s.reference == reference)
    }

    /// Get all symbols by component ID
    pub fn symbols_by_component(&self, component_id: ComponentId) -> Vec<&SchematicSymbol> {
        self.symbols
            .iter()
            .filter(|s| s.component_id == component_id)
            .collect()
    }
}

/// Complete schematic (multiple sheets)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schematic {
    /// Project name
    pub name: String,
    
    /// Sheets
    pub sheets: Vec<SchematicPage>,
    
    /// Title block info
    pub title: Option<String>,
    pub revision: Option<String>,
    pub author: Option<String>,
    pub date: Option<String>,
}

impl Schematic {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            sheets: vec![SchematicPage::new("Sheet 1")],
            title: None,
            revision: None,
            author: None,
            date: None,
        }
    }

    pub fn add_sheet(&mut self, sheet: SchematicPage) {
        self.sheets.push(sheet);
    }

    pub fn sheet_count(&self) -> usize {
        self.sheets.len()
    }

    pub fn current_sheet(&self) -> Option<&SchematicPage> {
        self.sheets.first()
    }

    pub fn current_sheet_mut(&mut self) -> Option<&mut SchematicPage> {
        self.sheets.first_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schematic_page() {
        let mut page = SchematicPage::new("Test Sheet");
        page.add_wire(Wire::from_coords(0.0, 0.0, 100.0, 0.0));
        assert_eq!(page.wires.len(), 1);
        assert!(page.wires[0].is_horizontal());
    }

    #[test]
    fn test_schematic() {
        let mut sch = Schematic::new("Test Project");
        assert_eq!(sch.sheet_count(), 1);
        sch.add_sheet(SchematicPage::new("Sheet 2"));
        assert_eq!(sch.sheet_count(), 2);
    }
}
