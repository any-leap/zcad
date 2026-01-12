//! Netlist management
//!
//! Defines nets, connections, and netlist operations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::schematic::SymbolInstanceId;

static NET_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Net ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NetId(pub u64);

impl NetId {
    pub fn new() -> Self {
        Self(NET_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for NetId {
    fn default() -> Self {
        Self::new()
    }
}

/// Net class (grouping with common properties)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetClass {
    /// Class name
    pub name: String,
    
    /// Description
    pub description: Option<String>,
    
    /// Track width (mm)
    pub track_width: f64,
    
    /// Via diameter (mm)
    pub via_diameter: f64,
    
    /// Via drill (mm)
    pub via_drill: f64,
    
    /// Clearance (mm)
    pub clearance: f64,
    
    /// Differential pair gap (mm)
    pub diff_pair_gap: Option<f64>,
}

impl Default for NetClass {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            description: None,
            track_width: 0.25,
            via_diameter: 0.8,
            via_drill: 0.4,
            clearance: 0.2,
            diff_pair_gap: None,
        }
    }
}

impl NetClass {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    pub fn power() -> Self {
        Self {
            name: "Power".to_string(),
            description: Some("Power nets".to_string()),
            track_width: 0.5,
            via_diameter: 1.0,
            via_drill: 0.5,
            clearance: 0.3,
            diff_pair_gap: None,
        }
    }

    pub fn differential(name: impl Into<String>, impedance: f64) -> Self {
        Self {
            name: name.into(),
            description: Some(format!("Differential pair {}Ω", impedance)),
            track_width: 0.15,
            via_diameter: 0.6,
            via_drill: 0.3,
            clearance: 0.15,
            diff_pair_gap: Some(0.15),
        }
    }
}

/// A pin reference (which component, which pin)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PinRef {
    /// Symbol instance ID
    pub symbol_id: SymbolInstanceId,
    
    /// Pin number/name
    pub pin: String,
}

impl PinRef {
    pub fn new(symbol_id: SymbolInstanceId, pin: impl Into<String>) -> Self {
        Self {
            symbol_id,
            pin: pin.into(),
        }
    }
}

/// A single net (electrical connection)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Net {
    /// Net ID
    pub id: NetId,
    
    /// Net name (auto-generated or user-defined)
    pub name: String,
    
    /// Connected pins
    pub pins: Vec<PinRef>,
    
    /// Net class
    pub class: Option<String>,
    
    /// Is this a power net?
    pub is_power: bool,
    
    /// Is this a ground net?
    pub is_ground: bool,
}

impl Net {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: NetId::new(),
            name: name.into(),
            pins: Vec::new(),
            class: None,
            is_power: false,
            is_ground: false,
        }
    }

    pub fn power(name: impl Into<String>) -> Self {
        let mut net = Self::new(name);
        net.is_power = true;
        net.class = Some("Power".to_string());
        net
    }

    pub fn ground() -> Self {
        let mut net = Self::new("GND");
        net.is_ground = true;
        net.class = Some("Power".to_string());
        net
    }

    pub fn add_pin(&mut self, pin_ref: PinRef) {
        if !self.pins.contains(&pin_ref) {
            self.pins.push(pin_ref);
        }
    }

    pub fn remove_pin(&mut self, pin_ref: &PinRef) {
        self.pins.retain(|p| p != pin_ref);
    }

    pub fn is_connected(&self, pin_ref: &PinRef) -> bool {
        self.pins.contains(pin_ref)
    }

    pub fn pin_count(&self) -> usize {
        self.pins.len()
    }

    /// A net with only one pin is "dangling" (incomplete connection)
    pub fn is_dangling(&self) -> bool {
        self.pins.len() == 1
    }

    /// A net with no pins is unused
    pub fn is_unused(&self) -> bool {
        self.pins.is_empty()
    }
}

/// Complete netlist
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Netlist {
    /// All nets
    nets: HashMap<NetId, Net>,
    
    /// Net classes
    classes: HashMap<String, NetClass>,
    
    /// Pin to net mapping
    pin_to_net: HashMap<PinRef, NetId>,
}

impl Netlist {
    pub fn new() -> Self {
        let mut netlist = Self {
            nets: HashMap::new(),
            classes: HashMap::new(),
            pin_to_net: HashMap::new(),
        };
        
        // Add default net class
        netlist.add_class(NetClass::default());
        netlist.add_class(NetClass::power());
        
        netlist
    }

    pub fn add_net(&mut self, net: Net) -> NetId {
        let id = net.id;
        
        // Update pin-to-net mapping
        for pin in &net.pins {
            self.pin_to_net.insert(pin.clone(), id);
        }
        
        self.nets.insert(id, net);
        id
    }

    pub fn remove_net(&mut self, net_id: NetId) -> Option<Net> {
        if let Some(net) = self.nets.remove(&net_id) {
            // Remove pin-to-net mappings
            for pin in &net.pins {
                self.pin_to_net.remove(pin);
            }
            Some(net)
        } else {
            None
        }
    }

    pub fn get_net(&self, net_id: NetId) -> Option<&Net> {
        self.nets.get(&net_id)
    }

    pub fn get_net_mut(&mut self, net_id: NetId) -> Option<&mut Net> {
        self.nets.get_mut(&net_id)
    }

    pub fn find_net_by_name(&self, name: &str) -> Option<&Net> {
        self.nets.values().find(|n| n.name == name)
    }

    pub fn get_net_for_pin(&self, pin_ref: &PinRef) -> Option<NetId> {
        self.pin_to_net.get(pin_ref).copied()
    }

    pub fn connect_pins(&mut self, pin1: PinRef, pin2: PinRef) -> NetId {
        let net1 = self.pin_to_net.get(&pin1).copied();
        let net2 = self.pin_to_net.get(&pin2).copied();

        match (net1, net2) {
            (Some(id1), Some(id2)) if id1 == id2 => {
                // Already connected
                id1
            }
            (Some(id1), Some(id2)) => {
                // Merge two nets
                let net2 = self.remove_net(id2).unwrap();
                if let Some(net1) = self.nets.get_mut(&id1) {
                    for pin in net2.pins {
                        net1.add_pin(pin.clone());
                        self.pin_to_net.insert(pin, id1);
                    }
                }
                id1
            }
            (Some(id), None) => {
                // Add pin2 to existing net
                if let Some(net) = self.nets.get_mut(&id) {
                    net.add_pin(pin2.clone());
                    self.pin_to_net.insert(pin2, id);
                }
                id
            }
            (None, Some(id)) => {
                // Add pin1 to existing net
                if let Some(net) = self.nets.get_mut(&id) {
                    net.add_pin(pin1.clone());
                    self.pin_to_net.insert(pin1, id);
                }
                id
            }
            (None, None) => {
                // Create new net
                let mut net = Net::new(format!("Net_{}", NET_ID_COUNTER.load(Ordering::Relaxed)));
                net.add_pin(pin1.clone());
                net.add_pin(pin2.clone());
                let id = net.id;
                self.pin_to_net.insert(pin1, id);
                self.pin_to_net.insert(pin2, id);
                self.nets.insert(id, net);
                id
            }
        }
    }

    pub fn add_class(&mut self, class: NetClass) {
        self.classes.insert(class.name.clone(), class);
    }

    pub fn get_class(&self, name: &str) -> Option<&NetClass> {
        self.classes.get(name)
    }

    pub fn net_count(&self) -> usize {
        self.nets.len()
    }

    pub fn nets(&self) -> impl Iterator<Item = &Net> {
        self.nets.values()
    }

    /// Get all nets that connect to a specific symbol
    pub fn nets_for_symbol(&self, symbol_id: SymbolInstanceId) -> Vec<&Net> {
        self.nets
            .values()
            .filter(|net| net.pins.iter().any(|p| p.symbol_id == symbol_id))
            .collect()
    }

    /// Check for dangling nets (incomplete connections)
    pub fn dangling_nets(&self) -> Vec<&Net> {
        self.nets.values().filter(|n| n.is_dangling()).collect()
    }

    /// Check for unused nets
    pub fn unused_nets(&self) -> Vec<&Net> {
        self.nets.values().filter(|n| n.is_unused()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_net_creation() {
        let net = Net::new("TestNet");
        assert_eq!(net.name, "TestNet");
        assert!(!net.is_power);
    }

    #[test]
    fn test_netlist_connect() {
        let mut netlist = Netlist::new();
        
        let pin1 = PinRef::new(SymbolInstanceId::new(), "1");
        let pin2 = PinRef::new(SymbolInstanceId::new(), "2");
        
        let net_id = netlist.connect_pins(pin1.clone(), pin2.clone());
        
        assert_eq!(netlist.get_net_for_pin(&pin1), Some(net_id));
        assert_eq!(netlist.get_net_for_pin(&pin2), Some(net_id));
    }
}
