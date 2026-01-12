//! 布局系统（Layout System）
//!
//! 实现 AutoCAD 风格的模型空间/图纸空间设计：
//! - **模型空间（Model Space）**：1:1 比例绘制实际几何图形
//! - **图纸空间（Paper Space/Layout）**：用于打印输出的虚拟图纸
//! - **视口（Viewport）**：在图纸空间中显示模型空间内容的"窗口"
//!
//! # 架构设计
//!
//! ```text
//! Document
//! ├── Model Space (entities)     <- 所有几何图形存放在这里
//! └── Layouts[]
//!     ├── Layout1
//!     │   ├── Paper Settings (A3, 横向...)
//!     │   ├── Paper Space Entities (图框、标题栏...)
//!     │   └── Viewports[]
//!     │       ├── Viewport1 (比例 1:100, 显示整体)
//!     │       └── Viewport2 (比例 1:10, 显示详图)
//!     └── Layout2
//!         └── ...
//! ```

mod types;
mod viewport;
mod layout;
mod manager;

pub use types::{LayoutId, ViewportId, PaperSize, PaperOrientation, ViewportStatus, SpaceType, STANDARD_SCALES};
pub use viewport::Viewport;
pub use layout::Layout;
pub use manager::LayoutManager;
