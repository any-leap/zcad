# zcad-libredwg

LibreDWG 的 Rust FFI 绑定，为 ZCAD 提供 DWG 文件读取支持。

## 前置要求

编译此 crate 需要：

1. **LibreDWG 库**（0.12+）
2. **LLVM/Clang**（用于 bindgen 生成绑定）

## 安装依赖

### Windows

**方法 1: 使用 vcpkg（推荐）**

```powershell
# 安装 vcpkg
git clone https://github.com/microsoft/vcpkg
cd vcpkg
.\bootstrap-vcpkg.bat
.\vcpkg integrate install

# 安装 libredwg
.\vcpkg install libredwg:x64-windows
```

**方法 2: 手动安装**

1. 从 [LibreDWG Releases](https://github.com/LibreDWG/libredwg/releases) 下载预编译版本
2. 解压到 `C:\libredwg`（或其他目录）
3. 设置环境变量：`LIBREDWG_DIR=C:\libredwg`

**安装 LLVM（必需）**

1. 从 [LLVM Releases](https://releases.llvm.org/) 下载并安装
2. 设置环境变量：`LIBCLANG_PATH=C:\Program Files\LLVM\bin`

### macOS

```bash
# 使用 Homebrew
brew install libredwg llvm

# 设置 LLVM 路径（如果需要）
export LIBCLANG_PATH=$(brew --prefix llvm)/lib
```

### Linux (Debian/Ubuntu)

```bash
# 安装 LLVM
sudo apt install llvm-dev libclang-dev

# LibreDWG 需要从源码编译（apt 仓库版本可能较旧）
sudo apt install libredwg-dev

# 或从源码编译最新版本
git clone https://github.com/LibreDWG/libredwg
cd libredwg
./autogen.sh
./configure
make
sudo make install
```

## 编译 ZCAD 并启用 DWG 支持

```bash
# 在项目根目录
cargo build --features dwg

# 或只编译 zcad-app 并启用 dwg
cargo build -p zcad-app --features dwg
```

## 使用方法

```rust
use zcad_libredwg::DwgFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dwg = DwgFile::open("drawing.dwg")?;
    
    println!("DWG Version: {}", dwg.version());
    println!("Layers: {:?}", dwg.layers());
    
    for entity in dwg.entities() {
        println!("{:?}", entity);
    }
    
    Ok(())
}
```

## 支持的实体类型

- LINE - 直线
- CIRCLE - 圆
- ARC - 圆弧
- LWPOLYLINE - 轻量多段线
- POLYLINE - 多段线
- POINT - 点
- TEXT - 单行文字
- MTEXT - 多行文字
- ELLIPSE - 椭圆
- SPLINE - 样条曲线
- INSERT - 块引用（部分支持）

## 注意事项

- LibreDWG 是 GNU 开源项目，对于某些新版本 DWG 格式可能支持有限
- 建议保留 DXF 导入作为备选方案
- 如果遇到解析问题，可以先将 DWG 转换为 DXF 格式

## 许可证

遵循项目主许可证（MIT OR Apache-2.0）

LibreDWG 本身使用 GPL-3.0 许可证。
