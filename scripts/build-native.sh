#!/bin/bash
# ZCAD 本地平台发布构建脚本

set -e

echo "🚀 开始构建 ZCAD 本地版本..."

# 检测操作系统
OS_TYPE=""
ARCH=$(uname -m)

if [[ "$OSTYPE" == "darwin"* ]]; then
    OS_TYPE="macos"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    OS_TYPE="linux"
elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
    OS_TYPE="windows"
else
    echo "❌ 不支持的操作系统: $OSTYPE"
    exit 1
fi

echo "📋 检测到系统: $OS_TYPE ($ARCH)"

# 编译
echo "🔨 编译 release 版本..."
cargo build --release

# 创建发布目录
DIST_DIR="dist/zcad-$OS_TYPE-$ARCH"
echo "📁 创建发布目录: $DIST_DIR"
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

# 复制文件
echo "📋 复制文件..."
if [[ "$OS_TYPE" == "windows" ]]; then
    cp target/release/zcad.exe "$DIST_DIR/"
else
    cp target/release/zcad "$DIST_DIR/"
fi

cp README.md "$DIST_DIR/"
cp LICENSE-MIT "$DIST_DIR/"
cp LICENSE-APACHE "$DIST_DIR/"

# 创建启动脚本（Linux/macOS）
if [[ "$OS_TYPE" != "windows" ]]; then
    cat > "$DIST_DIR/run.sh" << 'EOF'
#!/bin/bash
cd "$(dirname "$0")"
./zcad
EOF
    chmod +x "$DIST_DIR/run.sh"
    chmod +x "$DIST_DIR/zcad"
fi

# 压缩
echo "🗜️  压缩发布包..."
cd dist
ARCHIVE_NAME="zcad-$OS_TYPE-$ARCH"

if command -v tar &> /dev/null; then
    tar -czf "$ARCHIVE_NAME.tar.gz" "$ARCHIVE_NAME/"
    echo "  已创建: $ARCHIVE_NAME.tar.gz"
fi

if command -v zip &> /dev/null; then
    zip -r "$ARCHIVE_NAME.zip" "$ARCHIVE_NAME/"
    echo "  已创建: $ARCHIVE_NAME.zip"
fi

cd ..

echo "✅ 构建完成！"
echo ""
echo "输出文件："
echo "  - 目录: $DIST_DIR"
if [[ "$OS_TYPE" == "windows" ]]; then
    ls -lh "$DIST_DIR/zcad.exe" 2>/dev/null | awk '{print "  zcad.exe: " $5}' || true
else
    ls -lh "$DIST_DIR/zcad" 2>/dev/null | awk '{print "  zcad: " $5}' || true
fi

# 显示压缩包大小
for ext in tar.gz zip; do
    if [ -f "dist/$ARCHIVE_NAME.$ext" ]; then
        ls -lh "dist/$ARCHIVE_NAME.$ext" | awk '{print "  压缩包 ('"$ext"'): " $5}'
    fi
done

