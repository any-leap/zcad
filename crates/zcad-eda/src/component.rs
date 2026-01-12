//! Component library
//!
//! Defines electronic components with symbols, footprints, and metadata.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use zcad_core::math::Point2;

static COMPONENT_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Component ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComponentId(pub u64);

impl ComponentId {
    pub fn new() -> Self {
        Self(COMPONENT_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for ComponentId {
    fn default() -> Self {
        Self::new()
    }
}

/// Pin type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PinType {
    Input,
    Output,
    Bidirectional,
    Tristate,
    Passive,
    Power,
    Ground,
    Unspecified,
    OpenCollector,
    OpenEmitter,
    NoConnect,
}

/// Pin shape (for schematic symbol)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PinShape {
    Line,
    Inverted,
    Clock,
    InvertedClock,
    InputLow,
    OutputLow,
    FallingEdge,
    NonLogic,
}

/// A pin definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pin {
    /// Pin number/name
    pub number: String,
    
    /// Pin name
    pub name: String,
    
    /// Pin type
    pub pin_type: PinType,
    
    /// Pin shape
    pub shape: PinShape,
    
    /// Position on symbol (for schematic)
    pub symbol_position: Point2,
    
    /// Position on footprint (for PCB)
    pub footprint_position: Option<Point2>,
    
    /// Pin length (for schematic)
    pub length: f64,
    
    /// Rotation (degrees, 0/90/180/270)
    pub rotation: f64,
}

impl Pin {
    pub fn new(number: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            number: number.into(),
            name: name.into(),
            pin_type: PinType::Passive,
            shape: PinShape::Line,
            symbol_position: Point2::origin(),
            footprint_position: None,
            length: 100.0,
            rotation: 0.0,
        }
    }

    pub fn with_type(mut self, pin_type: PinType) -> Self {
        self.pin_type = pin_type;
        self
    }

    pub fn with_position(mut self, x: f64, y: f64) -> Self {
        self.symbol_position = Point2::new(x, y);
        self
    }
}

/// Symbol graphic element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SymbolGraphic {
    Rectangle {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        filled: bool,
    },
    Circle {
        cx: f64,
        cy: f64,
        radius: f64,
        filled: bool,
    },
    Arc {
        cx: f64,
        cy: f64,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
    },
    Line {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
    },
    Polyline {
        points: Vec<Point2>,
        closed: bool,
        filled: bool,
    },
    Text {
        x: f64,
        y: f64,
        text: String,
        size: f64,
    },
}

/// Schematic symbol definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    /// Symbol name
    pub name: String,
    
    /// Description
    pub description: Option<String>,
    
    /// Graphics (body)
    pub graphics: Vec<SymbolGraphic>,
    
    /// Pins
    pub pins: Vec<Pin>,
    
    /// Reference prefix (e.g., "R", "C", "U")
    pub reference_prefix: String,
    
    /// Unit count (for multi-unit symbols)
    pub units: u32,
}

impl Symbol {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            graphics: Vec::new(),
            pins: Vec::new(),
            reference_prefix: "U".to_string(),
            units: 1,
        }
    }

    pub fn add_pin(&mut self, pin: Pin) {
        self.pins.push(pin);
    }

    pub fn add_graphic(&mut self, graphic: SymbolGraphic) {
        self.graphics.push(graphic);
    }

    /// Create a simple resistor symbol
    pub fn resistor() -> Self {
        let mut symbol = Self::new("Resistor");
        symbol.reference_prefix = "R".to_string();
        symbol.description = Some("Resistor".to_string());

        // Add zigzag body
        symbol.graphics.push(SymbolGraphic::Polyline {
            points: vec![
                Point2::new(-100.0, 0.0),
                Point2::new(-80.0, 0.0),
                Point2::new(-70.0, 25.0),
                Point2::new(-50.0, -25.0),
                Point2::new(-30.0, 25.0),
                Point2::new(-10.0, -25.0),
                Point2::new(10.0, 25.0),
                Point2::new(30.0, -25.0),
                Point2::new(50.0, 25.0),
                Point2::new(70.0, -25.0),
                Point2::new(80.0, 0.0),
                Point2::new(100.0, 0.0),
            ],
            closed: false,
            filled: false,
        });

        symbol.pins.push(
            Pin::new("1", "1")
                .with_position(-100.0, 0.0)
                .with_type(PinType::Passive),
        );
        symbol.pins.push(
            Pin::new("2", "2")
                .with_position(100.0, 0.0)
                .with_type(PinType::Passive),
        );

        symbol
    }

    /// Create a simple capacitor symbol
    pub fn capacitor() -> Self {
        let mut symbol = Self::new("Capacitor");
        symbol.reference_prefix = "C".to_string();
        symbol.description = Some("Capacitor".to_string());

        // Two parallel plates
        symbol.graphics.push(SymbolGraphic::Line {
            x1: -100.0,
            y1: 0.0,
            x2: -10.0,
            y2: 0.0,
        });
        symbol.graphics.push(SymbolGraphic::Line {
            x1: -10.0,
            y1: -40.0,
            x2: -10.0,
            y2: 40.0,
        });
        symbol.graphics.push(SymbolGraphic::Line {
            x1: 10.0,
            y1: -40.0,
            x2: 10.0,
            y2: 40.0,
        });
        symbol.graphics.push(SymbolGraphic::Line {
            x1: 10.0,
            y1: 0.0,
            x2: 100.0,
            y2: 0.0,
        });

        symbol.pins.push(
            Pin::new("1", "1")
                .with_position(-100.0, 0.0)
                .with_type(PinType::Passive),
        );
        symbol.pins.push(
            Pin::new("2", "2")
                .with_position(100.0, 0.0)
                .with_type(PinType::Passive),
        );

        symbol
    }
}

/// Pad shape for footprints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PadShape {
    Circle,
    Rectangle,
    Oval,
    RoundedRectangle,
    Trapezoid,
    Custom,
}

/// Pad type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PadType {
    /// Through-hole pad
    ThroughHole,
    /// Surface mount pad
    Smd,
    /// Non-plated through-hole
    Npth,
    /// Connector pad
    Connector,
}

/// A footprint pad
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FootprintPad {
    /// Pad number (matches pin number)
    pub number: String,
    
    /// Position
    pub position: Point2,
    
    /// Pad type
    pub pad_type: PadType,
    
    /// Shape
    pub shape: PadShape,
    
    /// Size (width, height)
    pub size: (f64, f64),
    
    /// Drill size (for through-hole)
    pub drill: Option<f64>,
    
    /// Rotation (degrees)
    pub rotation: f64,
    
    /// Layers (for SMD: only top or bottom)
    pub layers: Vec<String>,
}

impl FootprintPad {
    pub fn smd(number: impl Into<String>, x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            number: number.into(),
            position: Point2::new(x, y),
            pad_type: PadType::Smd,
            shape: PadShape::Rectangle,
            size: (width, height),
            drill: None,
            rotation: 0.0,
            layers: vec!["F.Cu".to_string(), "F.Paste".to_string(), "F.Mask".to_string()],
        }
    }

    pub fn through_hole(number: impl Into<String>, x: f64, y: f64, pad_dia: f64, drill_dia: f64) -> Self {
        Self {
            number: number.into(),
            position: Point2::new(x, y),
            pad_type: PadType::ThroughHole,
            shape: PadShape::Circle,
            size: (pad_dia, pad_dia),
            drill: Some(drill_dia),
            rotation: 0.0,
            layers: vec!["*.Cu".to_string(), "*.Mask".to_string()],
        }
    }
}

/// PCB footprint definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Footprint {
    /// Footprint name
    pub name: String,
    
    /// Description
    pub description: Option<String>,
    
    /// Pads
    pub pads: Vec<FootprintPad>,
    
    /// Silkscreen graphics (outlines, text)
    pub silkscreen: Vec<SymbolGraphic>,
    
    /// Courtyard (component boundary)
    pub courtyard: Option<Vec<Point2>>,
    
    /// Reference position
    pub reference_position: Point2,
    
    /// Value position
    pub value_position: Point2,
}

impl Footprint {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            pads: Vec::new(),
            silkscreen: Vec::new(),
            courtyard: None,
            reference_position: Point2::new(0.0, -2.0),
            value_position: Point2::new(0.0, 2.0),
        }
    }

    pub fn add_pad(&mut self, pad: FootprintPad) {
        self.pads.push(pad);
    }

    /// Create a simple 0805 SMD resistor/capacitor footprint
    pub fn smd_0805() -> Self {
        let mut fp = Self::new("0805");
        fp.description = Some("SMD 0805 (2012 metric)".to_string());

        fp.add_pad(FootprintPad::smd("1", -0.95, 0.0, 1.0, 1.2));
        fp.add_pad(FootprintPad::smd("2", 0.95, 0.0, 1.0, 1.2));

        // Silkscreen outline
        fp.silkscreen.push(SymbolGraphic::Rectangle {
            x: -1.6,
            y: -0.8,
            width: 3.2,
            height: 1.6,
            filled: false,
        });

        fp
    }
}

/// A complete component (symbol + footprint + metadata)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    /// Component ID
    pub id: ComponentId,
    
    /// Component name
    pub name: String,
    
    /// Description
    pub description: Option<String>,
    
    /// Symbol
    pub symbol: Symbol,
    
    /// Footprint
    pub footprint: Footprint,
    
    /// Manufacturer
    pub manufacturer: Option<String>,
    
    /// Part number
    pub mpn: Option<String>,
    
    /// Datasheet URL
    pub datasheet: Option<String>,
    
    /// Default value (for R, C, L)
    pub value: Option<String>,
    
    /// Custom properties
    pub properties: HashMap<String, String>,
}

impl Component {
    pub fn new(name: impl Into<String>, symbol: Symbol, footprint: Footprint) -> Self {
        Self {
            id: ComponentId::new(),
            name: name.into(),
            description: None,
            symbol,
            footprint,
            manufacturer: None,
            mpn: None,
            datasheet: None,
            value: None,
            properties: HashMap::new(),
        }
    }

    /// Create a resistor component
    pub fn resistor(value: &str) -> Self {
        let mut comp = Self::new(
            format!("R_{}", value),
            Symbol::resistor(),
            Footprint::smd_0805(),
        );
        comp.value = Some(value.to_string());
        comp.description = Some("SMD Resistor 0805".to_string());
        comp
    }

    /// Create a capacitor component
    pub fn capacitor(value: &str) -> Self {
        let mut comp = Self::new(
            format!("C_{}", value),
            Symbol::capacitor(),
            Footprint::smd_0805(),
        );
        comp.value = Some(value.to_string());
        comp.description = Some("SMD Capacitor 0805".to_string());
        comp
    }
}

/// Component library
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ComponentLibrary {
    components: HashMap<String, Component>,
}

impl ComponentLibrary {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    pub fn add(&mut self, component: Component) {
        self.components.insert(component.name.clone(), component);
    }

    pub fn get(&self, name: &str) -> Option<&Component> {
        self.components.get(name)
    }

    pub fn names(&self) -> Vec<&str> {
        self.components.keys().map(|s| s.as_str()).collect()
    }

    pub fn len(&self) -> usize {
        self.components.len()
    }

    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resistor_symbol() {
        let symbol = Symbol::resistor();
        assert_eq!(symbol.reference_prefix, "R");
        assert_eq!(symbol.pins.len(), 2);
    }

    #[test]
    fn test_component_creation() {
        let comp = Component::resistor("10k");
        assert_eq!(comp.value, Some("10k".to_string()));
    }
}
