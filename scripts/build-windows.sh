#!/bin/bash
# ZCAD Windows 发布构建脚本

set -e

echo "🚀 开始构建 ZCAD Windows 版本..."

# 检查 Rust 工具链
if ! command -v rustup &> /dev/null; then
    echo "❌ 错误: 未找到 rustup，请先安装 Rust"
    exit 1
fi

# 添加 Windows 目标
echo "📦 添加 Windows 目标..."
rustup target add x86_64-pc-windows-gnu

# 检查 mingw-w64（macOS/Linux）
if [[ "$OSTYPE" == "darwin"* ]]; then
    if ! command -v x86_64-w64-mingw32-gcc &> /dev/null; then
        echo "⚠️  未找到 mingw-w64，尝试安装..."
        if command -v brew &> /dev/null; then
            brew install mingw-w64
        else
            echo "❌ 请手动安装 mingw-w64: brew install mingw-w64"
            exit 1
        fi
    fi
fi

# 编译
echo "🔨 编译 Windows 版本..."
cargo build --release --target x86_64-pc-windows-gnu

# 创建发布目录
DIST_DIR="dist/zcad-windows-x64"
echo "📁 创建发布目录: $DIST_DIR"
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

# 复制文件
echo "📋 复制文件..."
cp target/x86_64-pc-windows-gnu/release/zcad.exe "$DIST_DIR/"
cp README.md "$DIST_DIR/"
cp LICENSE-MIT "$DIST_DIR/"
cp LICENSE-APACHE "$DIST_DIR/"

# 创建 README（中文版）
cat > "$DIST_DIR/使用说明.txt" << 'EOF'
ZCAD - 开源 CAD 软件
==================

使用方法：
  双击 zcad.exe 运行

系统要求：
  - Windows 10 (1809+) 或 Windows 11
  - 支持 DirectX 12 的显卡

快捷键：
  文件操作：
    Ctrl+N  - 新建文档
    Ctrl+O  - 打开文件
    Ctrl+S  - 保存
    Ctrl+Shift+S - 另存为

  绘图工具：
    L - 直线
    C - 圆
    R - 矩形
    Space - 选择工具

  视图操作：
    Z - 缩放至全部
    G - 切换网格
    F8 - 切换正交模式
    鼠标滚轮 - 缩放
    鼠标中键拖动 - 平移

  编辑操作：
    Del - 删除选中对象
    Esc - 取消当前操作

许可证：
  MIT 或 Apache 2.0 双许可

问题反馈：
  https://github.com/zcad/zcad/issues
EOF

# 压缩
echo "🗜️  压缩发布包..."
cd dist
if command -v 7z &> /dev/null; then
    7z a -tzip zcad-windows-x64.zip zcad-windows-x64/
elif command -v zip &> /dev/null; then
    zip -r zcad-windows-x64.zip zcad-windows-x64/
else
    echo "⚠️  未找到压缩工具，跳过打包"
fi
cd ..

echo "✅ 构建完成！"
echo ""
echo "输出文件："
echo "  - 目录: $DIST_DIR"
echo "  - 压缩包: dist/zcad-windows-x64.zip"
echo ""
echo "文件大小："
ls -lh "$DIST_DIR/zcad.exe" | awk '{print "  zcad.exe: " $5}'
if [ -f "dist/zcad-windows-x64.zip" ]; then
    ls -lh "dist/zcad-windows-x64.zip" | awk '{print "  压缩包: " $5}'
fi

