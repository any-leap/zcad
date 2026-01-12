//! Architectural elements
//!
//! Walls, doors, windows, stairs, etc.

use serde::{Deserialize, Serialize};

/// Wall type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WallType {
    /// Solid wall
    Solid,
    /// Curtain wall
    Curtain,
    /// Partition
    Partition,
    /// Retaining wall
    Retaining,
}

/// Wall structure placeholder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wall {
    pub id: u64,
    pub name: String,
    pub wall_type: WallType,
    pub thickness: f64,
    pub height: f64,
}

/// Door type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DoorType {
    Single,
    Double,
    Sliding,
    Revolving,
    Folding,
}

/// Door placeholder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Door {
    pub id: u64,
    pub name: String,
    pub door_type: DoorType,
    pub width: f64,
    pub height: f64,
}

/// Window type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WindowType {
    Fixed,
    Casement,
    Sliding,
    Awning,
    Skylight,
}

/// Window placeholder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Window {
    pub id: u64,
    pub name: String,
    pub window_type: WindowType,
    pub width: f64,
    pub height: f64,
    pub sill_height: f64,
}

// More architectural elements would be defined here
