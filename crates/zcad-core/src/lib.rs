//! ZCAD 核心几何引擎
//!
//! 提供2D/3D几何图元、变换操作和空间查询功能。
//!
//! # 架构设计
//!
//! 采用 Entity-Component 模式：
//! - `Entity`: 唯一标识符
//! - `Geometry`: 几何数据（点、线、圆等）
//! - `Properties`: 视觉属性（颜色、线型、图层）
//!
//! # 示例
//!
//! ```rust
//! use zcad_core::prelude::*;
//!
//! // 创建一条线段
//! let line = Line::new(Point2::origin(), Point2::new(100.0, 50.0));
//!
//! // 计算长度
//! println!("Length: {}", line.length());
//! ```

pub mod entity;
pub mod geometry;
pub mod layer;
pub mod math;
pub mod properties;
pub mod spatial;
pub mod transform;

pub mod prelude {
    //! 常用类型的便捷导入
    pub use crate::entity::{Entity, EntityId};
    pub use crate::geometry::{Arc, Circle, Geometry, Line, Point, Polyline};
    pub use crate::layer::Layer;
    pub use crate::math::{Point2, Point3, Vector2, Vector3};
    pub use crate::properties::{Color, LineType, Properties};
    pub use crate::transform::Transform2D;
}

