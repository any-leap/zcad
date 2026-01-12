//! Design Rule Check (DRC) and Electrical Rule Check (ERC)
//!
//! Validates PCB and schematic designs against design rules.

use serde::{Deserialize, Serialize};
use zcad_core::math::Point2;

use crate::netlist::NetId;
use crate::pcb::{Board, ItemId, LayerId};
use crate::schematic::Schematic;

/// Violation severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// DRC violation type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DrcViolationType {
    /// Track too narrow
    TrackWidth {
        actual: f64,
        minimum: f64,
    },
    /// Clearance between items too small
    Clearance {
        item1: ItemId,
        item2: ItemId,
        actual: f64,
        minimum: f64,
    },
    /// Via too small
    ViaDrill {
        actual: f64,
        minimum: f64,
    },
    /// Annular ring too small
    AnnularRing {
        actual: f64,
        minimum: f64,
    },
    /// Unconnected copper (dangling)
    UnconnectedCopper {
        item: ItemId,
    },
    /// Net has no connection
    UnroutedNet {
        net: NetId,
    },
    /// Track stub (dead-end)
    TrackStub {
        item: ItemId,
    },
    /// Silkscreen over pad
    SilkOverPad {
        item: ItemId,
    },
    /// Courtyard overlap
    CourtyardOverlap {
        item1: ItemId,
        item2: ItemId,
    },
    /// Hole too close to edge
    HoleToEdge {
        item: ItemId,
        distance: f64,
        minimum: f64,
    },
    /// Acute angle in track
    AcuteAngle {
        item: ItemId,
        angle: f64,
    },
}

/// A DRC violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrcViolation {
    /// Violation type
    pub violation_type: DrcViolationType,
    
    /// Severity
    pub severity: Severity,
    
    /// Location
    pub location: Point2,
    
    /// Layer (if applicable)
    pub layer: Option<LayerId>,
    
    /// Description
    pub description: String,
}

impl DrcViolation {
    pub fn error(violation_type: DrcViolationType, location: Point2, description: impl Into<String>) -> Self {
        Self {
            violation_type,
            severity: Severity::Error,
            location,
            layer: None,
            description: description.into(),
        }
    }

    pub fn warning(violation_type: DrcViolationType, location: Point2, description: impl Into<String>) -> Self {
        Self {
            violation_type,
            severity: Severity::Warning,
            location,
            layer: None,
            description: description.into(),
        }
    }

    pub fn on_layer(mut self, layer: LayerId) -> Self {
        self.layer = Some(layer);
        self
    }
}

/// ERC violation type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErcViolationType {
    /// Unconnected pin
    UnconnectedPin {
        reference: String,
        pin: String,
    },
    /// Output driving output
    OutputConflict {
        net: String,
        pins: Vec<String>,
    },
    /// No driver for net
    NoDriver {
        net: String,
    },
    /// Power pins not connected
    PowerPinFloating {
        reference: String,
        pin: String,
    },
    /// Net with only one pin
    SinglePinNet {
        net: String,
    },
    /// Missing no-connect marker
    MissingNoConnect {
        reference: String,
        pin: String,
    },
}

/// An ERC violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErcViolation {
    /// Violation type
    pub violation_type: ErcViolationType,
    
    /// Severity
    pub severity: Severity,
    
    /// Description
    pub description: String,
}

impl ErcViolation {
    pub fn error(violation_type: ErcViolationType, description: impl Into<String>) -> Self {
        Self {
            violation_type,
            severity: Severity::Error,
            description: description.into(),
        }
    }

    pub fn warning(violation_type: ErcViolationType, description: impl Into<String>) -> Self {
        Self {
            violation_type,
            severity: Severity::Warning,
            description: description.into(),
        }
    }
}

/// DRC checker
pub struct DrcChecker {
    violations: Vec<DrcViolation>,
}

impl DrcChecker {
    pub fn new() -> Self {
        Self {
            violations: Vec::new(),
        }
    }

    /// Run DRC on a board
    pub fn check(&mut self, board: &Board) -> &[DrcViolation] {
        self.violations.clear();
        
        self.check_track_widths(board);
        self.check_clearances(board);
        self.check_via_sizes(board);
        // Add more checks...
        
        &self.violations
    }

    fn check_track_widths(&mut self, board: &Board) {
        for track in &board.tracks {
            if track.width < board.design_rules.min_track_width {
                self.violations.push(DrcViolation::error(
                    DrcViolationType::TrackWidth {
                        actual: track.width,
                        minimum: board.design_rules.min_track_width,
                    },
                    track.start,
                    format!(
                        "Track width {:.3}mm is less than minimum {:.3}mm",
                        track.width, board.design_rules.min_track_width
                    ),
                ));
            }
        }
    }

    fn check_clearances(&mut self, board: &Board) {
        // Simplified clearance check - just between tracks for now
        let tracks: Vec<_> = board.tracks.iter().collect();
        
        for i in 0..tracks.len() {
            for j in (i + 1)..tracks.len() {
                let t1 = tracks[i];
                let t2 = tracks[j];
                
                // Skip if same net
                if t1.net_id.is_some() && t1.net_id == t2.net_id {
                    continue;
                }
                
                // Skip if different layers
                if t1.layer != t2.layer {
                    continue;
                }
                
                // Simple point-to-point distance (not accurate for track segments)
                let dist = ((t1.start.x - t2.start.x).powi(2) + (t1.start.y - t2.start.y).powi(2)).sqrt();
                let min_dist = (t1.width + t2.width) / 2.0 + board.design_rules.min_clearance;
                
                if dist < min_dist {
                    self.violations.push(DrcViolation::warning(
                        DrcViolationType::Clearance {
                            item1: t1.id,
                            item2: t2.id,
                            actual: dist - (t1.width + t2.width) / 2.0,
                            minimum: board.design_rules.min_clearance,
                        },
                        t1.start,
                        format!(
                            "Clearance violation between tracks",
                        ),
                    ));
                }
            }
        }
    }

    fn check_via_sizes(&mut self, board: &Board) {
        for via in &board.vias {
            if via.drill < board.design_rules.min_via_drill {
                self.violations.push(DrcViolation::error(
                    DrcViolationType::ViaDrill {
                        actual: via.drill,
                        minimum: board.design_rules.min_via_drill,
                    },
                    via.position,
                    format!(
                        "Via drill {:.3}mm is less than minimum {:.3}mm",
                        via.drill, board.design_rules.min_via_drill
                    ),
                ));
            }
            
            let annular_ring = (via.diameter - via.drill) / 2.0;
            if annular_ring < board.design_rules.min_annular_ring {
                self.violations.push(DrcViolation::error(
                    DrcViolationType::AnnularRing {
                        actual: annular_ring,
                        minimum: board.design_rules.min_annular_ring,
                    },
                    via.position,
                    format!(
                        "Via annular ring {:.3}mm is less than minimum {:.3}mm",
                        annular_ring, board.design_rules.min_annular_ring
                    ),
                ));
            }
        }
    }

    pub fn violations(&self) -> &[DrcViolation] {
        &self.violations
    }

    pub fn error_count(&self) -> usize {
        self.violations.iter().filter(|v| v.severity == Severity::Error).count()
    }

    pub fn warning_count(&self) -> usize {
        self.violations.iter().filter(|v| v.severity == Severity::Warning).count()
    }
}

impl Default for DrcChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// ERC checker
pub struct ErcChecker {
    violations: Vec<ErcViolation>,
}

impl ErcChecker {
    pub fn new() -> Self {
        Self {
            violations: Vec::new(),
        }
    }

    /// Run ERC on a schematic
    pub fn check(&mut self, _schematic: &Schematic) -> &[ErcViolation] {
        self.violations.clear();
        
        // TODO: Implement ERC checks
        // - Check for unconnected pins
        // - Check for output conflicts
        // - Check for missing drivers
        // - Check for power pin connections
        
        &self.violations
    }

    pub fn violations(&self) -> &[ErcViolation] {
        &self.violations
    }
}

impl Default for ErcChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drc_checker() {
        let board = Board::new("Test", 100.0, 80.0);
        let mut checker = DrcChecker::new();
        let violations = checker.check(&board);
        assert!(violations.is_empty());
    }
}
