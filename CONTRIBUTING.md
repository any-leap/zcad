# 贡献指南

感谢你对 ZCAD 的关注！我们欢迎各种形式的贡献。

## 开发环境设置

### 前置要求

1. **Rust 工具链**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup default stable
   ```

2. **GPU 驱动**
   - Windows: Vulkan 或 DirectX 12
   - macOS: Metal (系统自带)
   - Linux: Vulkan

3. **可选工具**
   ```bash
   # 格式化
   rustup component add rustfmt
   
   # 代码检查
   rustup component add clippy
   ```

### 构建项目

```bash
# 克隆仓库
git clone https://github.com/zcad/zcad.git
cd zcad

# 开发构建
cargo build

# 优化构建
cargo build --release

# 运行
cargo run --release
```

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定 crate 的测试
cargo test -p zcad-core

# 运行带输出的测试
cargo test -- --nocapture
```

## 代码规范

### 格式化

提交前请运行：

```bash
cargo fmt
```

### Lint

确保没有警告：

```bash
cargo clippy -- -D warnings
```

### 文档

所有公开 API 都应该有文档：

```rust
/// 创建新的线段
///
/// # 参数
///
/// * `start` - 起点坐标
/// * `end` - 终点坐标
///
/// # 示例
///
/// ```rust
/// let line = Line::new(Point2::origin(), Point2::new(10.0, 10.0));
/// ```
pub fn new(start: Point2, end: Point2) -> Self {
    // ...
}
```

## 提交规范

使用约定式提交（Conventional Commits）：

```
类型(范围): 描述

[可选的正文]

[可选的页脚]
```

类型：
- `feat`: 新功能
- `fix`: 修复 bug
- `docs`: 文档更新
- `style`: 代码格式（不影响功能）
- `refactor`: 重构
- `test`: 测试
- `chore`: 构建/工具更新

示例：
```
feat(geometry): 添加椭圆支持

实现了椭圆的创建、渲染和序列化功能。

Closes #123
```

## Pull Request 流程

1. Fork 仓库
2. 创建功能分支：`git checkout -b feature/amazing-feature`
3. 提交更改：`git commit -m 'feat: add amazing feature'`
4. 推送分支：`git push origin feature/amazing-feature`
5. 创建 Pull Request

### PR 检查清单

- [ ] 代码通过 `cargo fmt`
- [ ] 代码通过 `cargo clippy`
- [ ] 所有测试通过 `cargo test`
- [ ] 添加了必要的测试
- [ ] 更新了相关文档
- [ ] 提交信息符合规范

## 项目结构

```
zcad/
├── crates/
│   ├── zcad-core/       # 核心几何引擎
│   ├── zcad-renderer/   # GPU渲染器
│   ├── zcad-file/       # 文件格式处理
│   ├── zcad-ui/         # 用户界面组件
│   └── zcad-app/        # 主应用程序
├── docs/                # 文档
└── examples/            # 示例文件
```

## 问题报告

报告问题时请包含：

1. ZCAD 版本
2. 操作系统和版本
3. GPU 型号和驱动版本
4. 复现步骤
5. 预期行为
6. 实际行为
7. 相关日志或截图

## 许可证

贡献的代码将使用与项目相同的双重许可：MIT 或 Apache-2.0。

