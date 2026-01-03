# ZCAD 文件格式规范

## 版本

当前版本：1

## 概述

ZCAD使用基于SQLite的单文件格式（`.zcad`），这种设计带来以下优势：

- **单文件**：易于分发和管理
- **事务支持**：操作原子性，不会产生损坏的文件
- **增量保存**：只写入变化的数据
- **查询能力**：可以使用标准SQL工具查看和编辑
- **跨平台**：SQLite在所有平台上都有良好支持

## 数据库架构

### metadata 表

存储文档级别的元数据。

```sql
CREATE TABLE metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

预定义的key：

| Key | 类型 | 描述 |
|-----|------|------|
| format_version | Integer | 文件格式版本号 |
| document | JSON | 文档元数据 |

document JSON结构：

```json
{
    "id": "uuid-v4",
    "title": "Document Title",
    "author": "Author Name",
    "created_at": "2024-01-01T00:00:00Z",
    "modified_at": "2024-01-01T00:00:00Z",
    "format_version": 1,
    "units": "mm",
    "custom_properties": {}
}
```

### layers 表

存储图层定义。

```sql
CREATE TABLE layers (
    id INTEGER PRIMARY KEY,
    generation INTEGER NOT NULL,
    name TEXT NOT NULL UNIQUE,
    data TEXT NOT NULL  -- JSON
);
```

data JSON结构：

```json
{
    "id": {"id": 1, "generation": 0},
    "name": "Layer1",
    "color": {"r": 255, "g": 255, "b": 255, "a": 255},
    "line_type": "Continuous",
    "line_weight": "Default",
    "visible": true,
    "locked": false,
    "frozen": false,
    "plottable": true,
    "description": ""
}
```

### entities 表

存储所有几何实体。

```sql
CREATE TABLE entities (
    id INTEGER PRIMARY KEY,
    generation INTEGER NOT NULL,
    layer_id INTEGER NOT NULL,
    geometry_type TEXT NOT NULL,
    geometry_data BLOB NOT NULL,  -- JSON (可选压缩)
    properties_data TEXT NOT NULL,  -- JSON
    visible INTEGER NOT NULL DEFAULT 1,
    locked INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_entities_layer ON entities(layer_id);
CREATE INDEX idx_entities_type ON entities(geometry_type);
```

#### geometry_type 值

| 类型 | 描述 |
|------|------|
| Point | 点 |
| Line | 线段 |
| Circle | 圆 |
| Arc | 圆弧 |
| Polyline | 多段线 |
| Ellipse | 椭圆 |
| Spline | 样条曲线 |
| Text | 文字 |
| Hatch | 填充 |

#### geometry_data 结构

**Point:**
```json
{
    "Point": {
        "position": {"x": 0.0, "y": 0.0}
    }
}
```

**Line:**
```json
{
    "Line": {
        "start": {"x": 0.0, "y": 0.0},
        "end": {"x": 100.0, "y": 100.0}
    }
}
```

**Circle:**
```json
{
    "Circle": {
        "center": {"x": 50.0, "y": 50.0},
        "radius": 25.0
    }
}
```

**Arc:**
```json
{
    "Arc": {
        "center": {"x": 50.0, "y": 50.0},
        "radius": 25.0,
        "start_angle": 0.0,
        "end_angle": 1.5707963267948966
    }
}
```

**Polyline:**
```json
{
    "Polyline": {
        "vertices": [
            {"point": {"x": 0.0, "y": 0.0}, "bulge": 0.0},
            {"point": {"x": 100.0, "y": 0.0}, "bulge": 0.5},
            {"point": {"x": 100.0, "y": 100.0}, "bulge": 0.0}
        ],
        "closed": true
    }
}
```

#### properties_data 结构

```json
{
    "color": {"r": 255, "g": 0, "b": 0, "a": 255},
    "line_type": "Continuous",
    "line_weight": "Default",
    "transparency": 0
}
```

颜色特殊值：
- `{"r": 0, "g": 0, "b": 0, "a": 0}` - ByLayer
- `{"r": 0, "g": 0, "b": 0, "a": 1}` - ByBlock

### views 表

存储保存的视图。

```sql
CREATE TABLE views (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    center_x REAL NOT NULL,
    center_y REAL NOT NULL,
    zoom REAL NOT NULL
);
```

## 文件操作

### 创建新文件

1. 创建SQLite数据库
2. 执行架构创建语句
3. 插入默认元数据
4. 插入默认图层（图层0）

### 保存文件

1. 开始事务
2. 更新元数据
3. 同步图层（删除、更新、插入）
4. 同步实体（删除、更新、插入）
5. 同步视图
6. 提交事务
7. 执行VACUUM优化（可选）

### 加载文件

1. 验证格式版本
2. 读取元数据
3. 读取所有图层
4. 读取所有实体
5. 读取所有视图
6. 重建空间索引

## 压缩

对于大型几何数据（如包含大量顶点的多段线），可以使用zlib压缩：

```
geometry_data = zlib.compress(json.dumps(geometry))
```

压缩标志存储在geometry_type中：
- `Polyline` - 未压缩
- `Polyline:z` - zlib压缩

## 版本兼容性

- 低版本软件无法打开高版本文件
- 高版本软件可以打开低版本文件（向后兼容）
- 版本升级时自动迁移数据

## 与DXF的互操作

ZCAD支持导入和导出DXF格式。

### 导入映射

| DXF实体 | ZCAD几何 |
|---------|----------|
| LINE | Line |
| CIRCLE | Circle |
| ARC | Arc |
| LWPOLYLINE | Polyline |
| POLYLINE | Polyline |
| POINT | Point |

### 导出映射

所有ZCAD几何都可以导出为对应的DXF实体。

### 颜色映射

使用AutoCAD颜色索引(ACI)与RGB之间的标准映射。

| ACI | 颜色 | RGB |
|-----|------|-----|
| 1 | Red | #FF0000 |
| 2 | Yellow | #FFFF00 |
| 3 | Green | #00FF00 |
| 4 | Cyan | #00FFFF |
| 5 | Blue | #0000FF |
| 6 | Magenta | #FF00FF |
| 7 | White | #FFFFFF |
| 256 | ByLayer | - |
| 0 | ByBlock | - |

