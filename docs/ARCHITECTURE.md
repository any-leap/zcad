# ZCAD 架构设计文档

## 1. 概述

ZCAD 是一个现代化的开源CAD系统，旨在解决传统CAD软件的性能和开放性问题。

### 1.1 设计目标

1. **极致性能**：充分利用多核CPU和GPU，即使处理百万级实体也能流畅操作
2. **开放格式**：原生格式完全开放，支持标准格式互操作
3. **跨平台**：支持Windows、macOS、Linux，未来支持Web
4. **可扩展**：插件系统和脚本支持

### 1.2 技术选型

| 技术 | 选择 | 理由 |
|------|------|------|
| 编程语言 | Rust | 内存安全、零成本抽象、优秀的并发模型 |
| 渲染后端 | wgpu/WebGPU | 跨平台GPU加速，支持Vulkan/Metal/DX12 |
| UI框架 | egui | 即时模式GUI，适合工程软件，易于定制 |
| 几何库 | nalgebra + parry | Rust生态中最成熟的几何库 |
| 文件格式 | SQLite + JSON/FlatBuffers | 高效、开放、易于调试 |

## 2. 系统架构

```
┌─────────────────────────────────────────────────────────────┐
│                        ZCAD Application                       │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   zcad-ui   │  │zcad-renderer│  │     zcad-file       │  │
│  │   (egui)    │  │   (wgpu)    │  │ (SQLite + DXF)      │  │
│  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘  │
│         │                │                     │              │
│  ┌──────┴────────────────┴─────────────────────┴──────────┐  │
│  │                      zcad-core                          │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐   │  │
│  │  │ Geometry │ │ Entity   │ │ Layer    │ │ Spatial  │   │  │
│  │  │ Engine   │ │ System   │ │ Manager  │ │ Index    │   │  │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘   │  │
│  └─────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## 3. 核心模块

### 3.1 zcad-core（几何引擎）

#### 3.1.1 几何图元

支持的2D图元：
- **Point** - 点
- **Line** - 线段
- **Circle** - 圆
- **Arc** - 圆弧
- **Polyline** - 多段线（支持弧线段）
- **Ellipse** - 椭圆（计划中）
- **Spline** - 样条曲线（计划中）
- **Text** - 文字（计划中）
- **Hatch** - 填充（计划中）

#### 3.1.2 实体系统

采用Entity-Component模式：

```rust
pub struct Entity {
    pub id: EntityId,           // 唯一标识符
    pub geometry: Geometry,     // 几何数据
    pub properties: Properties, // 视觉属性
    pub layer_id: EntityId,     // 所属图层
    pub visible: bool,          // 可见性
    pub locked: bool,           // 锁定状态
}
```

EntityId采用生成式设计，支持撤销/重做时的实体复用：

```rust
pub struct EntityId {
    pub id: u64,        // 唯一ID
    pub generation: u32, // 代数
}
```

#### 3.1.3 空间索引

使用网格空间索引（可升级为R-tree）：

```rust
pub struct SpatialIndex {
    cell_size: f64,
    grid: HashMap<(i64, i64), Vec<EntityId>>,
    bboxes: HashMap<EntityId, BoundingBox2>,
}
```

支持的查询：
- `query_rect()` - 矩形范围查询
- `query_point()` - 点击测试
- `query_nearest()` - 最近邻查询

#### 3.1.4 变换操作

2D仿射变换支持：
- 平移 (Translation)
- 旋转 (Rotation)
- 缩放 (Scale)
- 镜像 (Mirror)
- 组合变换

### 3.2 zcad-renderer（GPU渲染器）

#### 3.2.1 渲染管线

```
几何数据 → 顶点生成 → GPU缓冲区 → 着色器 → 屏幕
           (CPU)       (GPU)     (GPU)
```

关键优化：
1. **批量渲染**：相同类型的几何体合并到一个Draw Call
2. **LOD**：远距离自动降低细节
3. **视锥剔除**：只渲染可见范围内的实体
4. **增量更新**：只更新变化的部分

#### 3.2.2 着色器

使用WGSL（WebGPU Shading Language）：

```wgsl
struct CameraUniform {
    view_proj: mat4x4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(in.position, 0.0, 1.0);
    out.color = in.color;
    return out;
}
```

### 3.3 zcad-file（文件格式）

#### 3.3.1 原生格式 (.zcad)

基于SQLite的单文件格式：

```
zcad文件结构
├── metadata    - 文档元数据（JSON）
├── layers      - 图层定义
├── entities    - 几何实体
├── views       - 保存的视图
└── history     - 版本历史（可选）
```

数据库架构：

```sql
CREATE TABLE metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE entities (
    id INTEGER PRIMARY KEY,
    generation INTEGER NOT NULL,
    layer_id INTEGER NOT NULL,
    geometry_type TEXT NOT NULL,
    geometry_data BLOB NOT NULL,
    properties_data TEXT NOT NULL,
    visible INTEGER DEFAULT 1,
    locked INTEGER DEFAULT 0
);

CREATE TABLE layers (
    id INTEGER PRIMARY KEY,
    generation INTEGER NOT NULL,
    name TEXT NOT NULL UNIQUE,
    data TEXT NOT NULL
);
```

#### 3.3.2 DXF互操作

使用 `dxf` crate 实现AutoCAD DXF格式的读写。

支持的实体类型：
- LINE
- CIRCLE
- ARC
- LWPOLYLINE / POLYLINE
- POINT

### 3.4 zcad-ui（用户界面）

#### 3.4.1 界面布局

```
┌────────────────────────────────────────────────────────────┐
│ File  Edit  View  Draw  Modify  Help                       │
├────────────────────────────────────────────────────────────┤
│ [New][Open][Save] | [Select][Line][Circle][Arc][Poly] | ... │
├────────────┬───────────────────────────────┬───────────────┤
│            │                               │               │
│ Properties │     Drawing Canvas            │   Layers      │
│   Panel    │                               │   Panel       │
│            │                               │               │
├────────────┴───────────────────────────────┴───────────────┤
│ Command: _                         X: 123.45  Y: 67.89      │
└────────────────────────────────────────────────────────────┘
```

#### 3.4.2 命令系统

支持类AutoCAD的命令行输入：

| 命令 | 快捷键 | 功能 |
|------|--------|------|
| LINE | L | 绘制线段 |
| CIRCLE | C | 绘制圆 |
| ARC | A | 绘制圆弧 |
| POLYLINE | P/PL | 绘制多段线 |
| MOVE | M | 移动 |
| COPY | CO | 复制 |
| ROTATE | RO | 旋转 |
| SCALE | SC | 缩放 |
| MIRROR | MI | 镜像 |
| ERASE | E | 删除 |
| UNDO | U | 撤销 |

## 4. 性能优化策略

### 4.1 渲染优化

1. **GPU加速**：所有几何体在GPU上渲染
2. **实例化渲染**：相同几何体使用GPU实例化
3. **视锥剔除**：使用空间索引快速筛选可见实体
4. **LOD系统**：远距离简化几何体

### 4.2 数据结构优化

1. **空间索引**：O(log n) 复杂度的空间查询
2. **增量更新**：只更新变化的实体
3. **内存池**：减少内存分配开销
4. **并行计算**：使用rayon进行多线程计算

### 4.3 IO优化

1. **异步IO**：文件操作不阻塞主线程
2. **增量保存**：只保存变化的数据
3. **压缩存储**：可选的数据压缩

## 5. 未来规划

### Phase 1: 基础功能 (v0.1)
- [x] 核心几何库
- [x] GPU渲染器
- [x] 基础UI
- [x] 文件格式

### Phase 2: 完整CAD功能 (v0.2)
- [ ] 完整的绘图工具
- [ ] 编辑命令
- [ ] 图层管理
- [ ] 捕捉系统
- [ ] 撤销/重做

### Phase 3: 高级功能 (v0.3)
- [ ] 块和外部参照
- [ ] 标注系统
- [ ] 打印和导出
- [ ] 约束求解器

### Phase 4: 生态建设 (v1.0)
- [ ] 插件系统
- [ ] 脚本支持（Lua/Python）
- [ ] 在线协作
- [ ] WebAssembly版本

## 6. 与传统CAD的对比

| 特性 | 传统CAD | ZCAD |
|------|---------|------|
| 渲染 | CPU软件渲染 | GPU加速 |
| 多线程 | 有限 | 充分利用 |
| 大文件处理 | 卡顿 | 流畅 |
| 文件格式 | 专有/封闭 | 开放 |
| 价格 | 昂贵 | 免费开源 |
| 平台 | Windows为主 | 全平台 |
| 定制性 | 有限 | 完全可定制 |

