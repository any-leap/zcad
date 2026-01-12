//! ZCAD GPU渲染器
//!
//! 基于wgpu实现的高性能2D/3D CAD渲染器。
//!
//! # 特点
//!
//! - **GPU加速**：所有几何体在GPU上渲染
//! - **批量渲染**：相同类型的几何体批量处理
//! - **LOD支持**：远距离自动简化
//! - **抗锯齿**：MSAA和线条抗锯齿
//! - **3D支持**：网格渲染、光照、深度测试

// 2D rendering
pub mod camera;
pub mod compute;
pub mod pipeline;
pub mod renderer;
pub mod tile;
pub mod vertex;

// 3D rendering
pub mod camera3d;
pub mod mesh_renderer;

// 2D exports
pub use camera::Camera2D;
pub use compute::{BooleanOp, ComputeShader};
pub use renderer::Renderer;
pub use tile::{Tile, TileManager};

// 3D exports
pub use camera3d::{Camera3D, Camera3DUniform};
pub use mesh_renderer::{
    create_depth_texture, Line3DPipeline, Line3DVertex, MeshPipeline, MeshVertex, Render3DError,
};
