//! 对象捕捉系统
//!
//! 参考 LibreCAD 的设计，实现 CAD 标准的对象捕捉功能。
//!
//! 支持的捕捉类型：
//! - 端点 (Endpoint)
//! - 中点 (Midpoint)
//! - 圆心 (Center)
//! - 交点 (Intersection)
//! - 垂足 (Perpendicular)
//! - 切点 (Tangent)
//! - 最近点 (Nearest)
//! - 网格点 (Grid)

mod types;
mod config;
mod engine;
mod finders;
mod intersection;
mod polar;

pub use types::{SnapType, SnapPoint};
pub use config::{SnapConfig, SnapMask};
pub use engine::SnapEngine;
pub use finders::SnapHelpers;
pub use intersection::IntersectionFinder;
pub use polar::PolarSnap;
