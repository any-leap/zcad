//! 点

use crate::math::{BoundingBox2, Point2};
use serde::{Deserialize, Serialize};

/// 点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    pub position: Point2,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            position: Point2::new(x, y),
        }
    }

    pub fn from_point2(position: Point2) -> Self {
        Self { position }
    }

    pub fn bounding_box(&self) -> BoundingBox2 {
        BoundingBox2::new(self.position, self.position)
    }
}
