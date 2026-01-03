# ZCAD - 下一代开源CAD系统

<div align="center">

![ZCAD Logo](docs/assets/logo.svg)

**快速 • 开放 • 现代**

[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.83+-orange.svg)](https://www.rust-lang.org/)

</div>

## 🎯 愿景

ZCAD 致力于成为传统CAD软件的现代替代品，解决以下痛点：

- **性能问题**：传统CAD在大图形下卡顿严重，即使是高端硬件也无法流畅操作
- **封闭生态**：专有格式、昂贵授权、功能受限
- **糟糕的对象处理**：如xclip爆炸产生大量无用对象

## ✨ 核心特性

### 🚀 极致性能
- **GPU加速渲染**：基于WebGPU/wgpu，充分利用现代GPU
- **多线程架构**：几何计算、渲染、UI完全并行
- **智能LOD**：远距离自动简化，保持流畅
- **增量更新**：只重绘变化的部分

### 📐 精确的几何内核
- 自研2D/3D几何引擎
- 精确的布尔运算
- 智能的对象爆炸和分解
- 约束求解器

### 📁 开放的文件格式
- `.zcad` 原生格式（基于SQLite）
- 完整的DXF读写支持
- 可扩展的插件系统

### 🎨 现代UI
- 深色主题，护眼设计
- 自定义工具栏和快捷键
- 命令行界面（类AutoCAD）
- 多视口支持

## 🛠️ 技术栈

| 组件 | 技术 | 说明 |
|------|------|------|
| 语言 | Rust | 安全、高性能 |
| 渲染 | wgpu | 跨平台GPU加速 |
| UI | egui | 即时模式GUI |
| 几何 | nalgebra + parry | 向量/矩阵 + 碰撞检测 |
| 存储 | SQLite + FlatBuffers | 高效的数据持久化 |

## 📦 项目结构

```
zcad/
├── crates/
│   ├── zcad-core/       # 核心几何引擎
│   ├── zcad-renderer/   # GPU渲染器
│   ├── zcad-file/       # 文件格式处理
│   ├── zcad-ui/         # 用户界面组件
│   └── zcad-app/        # 主应用程序
├── docs/                 # 文档
└── examples/            # 示例文件
```

## 🚀 快速开始

### 前置要求

- Rust 1.83+
- 支持Vulkan/Metal/DX12的GPU

### 构建

```bash
# 克隆仓库
git clone https://github.com/zcad/zcad.git
cd zcad

# 构建并运行
cargo run --release
```

## 🗺️ 路线图

### Phase 1: 基础框架 (当前)
- [ ] 核心几何库（点、线、圆、弧、多段线）
- [ ] GPU渲染管线
- [ ] 基础UI框架
- [ ] 文件格式规范

### Phase 2: 基础功能
- [ ] 绘图命令（LINE, CIRCLE, ARC, PLINE等）
- [ ] 编辑命令（MOVE, COPY, ROTATE, SCALE等）
- [ ] 图层管理
- [ ] 捕捉和追踪

### Phase 3: 高级功能
- [ ] 块和外部参照
- [ ] 标注系统
- [ ] 打印/导出
- [ ] DXF互操作

### Phase 4: 生态建设
- [ ] 插件系统
- [ ] 脚本支持（Lua/Python）
- [ ] 在线协作

## 📄 许可证

双重许可：MIT 或 Apache-2.0，由您选择。

## 🤝 贡献

欢迎贡献！请参阅 [CONTRIBUTING.md](CONTRIBUTING.md) 了解详情。

