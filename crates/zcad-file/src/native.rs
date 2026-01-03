//! ZCAD原生文件格式（.zcad）
//!
//! 基于SQLite的单文件格式，支持：
//! - 增量保存
//! - 压缩存储
//! - 版本历史（可选）

use crate::document::{Document, DocumentMetadata, SavedView};
use crate::error::FileError;
use rusqlite::{params, Connection};
use std::path::Path;
use zcad_core::entity::{Entity, EntityId};
use zcad_core::geometry::Geometry;
use zcad_core::layer::Layer;
use zcad_core::properties::Properties;

/// 当前文件格式版本
const FORMAT_VERSION: u32 = 1;

/// 创建数据库架构
fn create_schema(conn: &Connection) -> Result<(), FileError> {
    conn.execute_batch(
        r#"
        -- 元数据表
        CREATE TABLE IF NOT EXISTS metadata (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        -- 图层表
        CREATE TABLE IF NOT EXISTS layers (
            id INTEGER PRIMARY KEY,
            generation INTEGER NOT NULL,
            name TEXT NOT NULL UNIQUE,
            data TEXT NOT NULL
        );

        -- 实体表
        CREATE TABLE IF NOT EXISTS entities (
            id INTEGER PRIMARY KEY,
            generation INTEGER NOT NULL,
            layer_id INTEGER NOT NULL,
            geometry_type TEXT NOT NULL,
            geometry_data BLOB NOT NULL,
            properties_data TEXT NOT NULL,
            visible INTEGER NOT NULL DEFAULT 1,
            locked INTEGER NOT NULL DEFAULT 0
        );

        -- 视图表
        CREATE TABLE IF NOT EXISTS views (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            center_x REAL NOT NULL,
            center_y REAL NOT NULL,
            zoom REAL NOT NULL
        );

        -- 创建索引
        CREATE INDEX IF NOT EXISTS idx_entities_layer ON entities(layer_id);
        CREATE INDEX IF NOT EXISTS idx_entities_type ON entities(geometry_type);
        "#,
    )?;

    Ok(())
}

/// 保存文档到文件
pub fn save(document: &Document, path: &Path) -> Result<(), FileError> {
    let conn = Connection::open(path)?;

    // 创建架构
    create_schema(&conn)?;

    // 开始事务
    conn.execute("BEGIN TRANSACTION", [])?;

    // 保存元数据
    save_metadata(&conn, &document.metadata)?;

    // 清空并保存图层
    conn.execute("DELETE FROM layers", [])?;
    for layer in document.layers.all_layers() {
        save_layer(&conn, layer)?;
    }

    // 清空并保存实体
    conn.execute("DELETE FROM entities", [])?;
    for entity in document.all_entities() {
        save_entity(&conn, entity)?;
    }

    // 清空并保存视图
    conn.execute("DELETE FROM views", [])?;
    for view in &document.views {
        save_view(&conn, view)?;
    }

    // 提交事务
    conn.execute("COMMIT", [])?;

    // 优化数据库
    conn.execute("VACUUM", [])?;

    Ok(())
}

fn save_metadata(conn: &Connection, metadata: &DocumentMetadata) -> Result<(), FileError> {
    let json = serde_json::to_string(metadata)?;
    conn.execute(
        "INSERT OR REPLACE INTO metadata (key, value) VALUES ('document', ?)",
        params![json],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO metadata (key, value) VALUES ('format_version', ?)",
        params![FORMAT_VERSION.to_string()],
    )?;
    Ok(())
}

fn save_layer(conn: &Connection, layer: &Layer) -> Result<(), FileError> {
    let data = serde_json::to_string(layer)?;
    conn.execute(
        "INSERT INTO layers (id, generation, name, data) VALUES (?, ?, ?, ?)",
        params![layer.id.id as i64, layer.id.generation, &layer.name, &data],
    )?;
    Ok(())
}

fn save_entity(conn: &Connection, entity: &Entity) -> Result<(), FileError> {
    let geometry_type = entity.geometry.type_name();
    let geometry_data = serde_json::to_vec(&entity.geometry)?;
    let properties_data = serde_json::to_string(&entity.properties)?;

    conn.execute(
        "INSERT INTO entities (id, generation, layer_id, geometry_type, geometry_data, properties_data, visible, locked)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            entity.id.id as i64,
            entity.id.generation,
            entity.layer_id.id as i64,
            geometry_type,
            &geometry_data,
            &properties_data,
            entity.visible as i32,
            entity.locked as i32,
        ],
    )?;

    Ok(())
}

fn save_view(conn: &Connection, view: &SavedView) -> Result<(), FileError> {
    conn.execute(
        "INSERT INTO views (name, center_x, center_y, zoom) VALUES (?, ?, ?, ?)",
        params![&view.name, view.center_x, view.center_y, view.zoom],
    )?;
    Ok(())
}

/// 从文件加载文档
pub fn load(path: &Path) -> Result<Document, FileError> {
    let conn = Connection::open(path)?;

    // 检查格式版本
    let version: String = conn.query_row(
        "SELECT value FROM metadata WHERE key = 'format_version'",
        [],
        |row| row.get(0),
    )?;

    let version: u32 = version
        .parse()
        .map_err(|_| FileError::InvalidFormat("Invalid version".to_string()))?;

    if version > FORMAT_VERSION {
        return Err(FileError::UnsupportedVersion(format!(
            "File version {} is newer than supported version {}",
            version, FORMAT_VERSION
        )));
    }

    let mut document = Document::new();

    // 加载元数据
    let metadata_json: String = conn.query_row(
        "SELECT value FROM metadata WHERE key = 'document'",
        [],
        |row| row.get(0),
    )?;
    document.metadata = serde_json::from_str(&metadata_json)?;

    // 加载图层
    let mut stmt = conn.prepare("SELECT data FROM layers ORDER BY id")?;
    let layers: Vec<Layer> = stmt
        .query_map([], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        })?
        .filter_map(|r| r.ok())
        .filter_map(|data| serde_json::from_str(&data).ok())
        .collect();

    // 重建图层管理器
    document.layers = zcad_core::layer::LayerManager::new();
    for layer in layers.into_iter().skip(1) {
        // 跳过默认图层0
        document.layers.add_layer(layer);
    }

    // 加载实体
    let mut stmt = conn.prepare(
        "SELECT id, generation, layer_id, geometry_data, properties_data, visible, locked FROM entities",
    )?;

    let entities: Vec<Entity> = stmt
        .query_map([], |row| {
            let id: i64 = row.get(0)?;
            let generation: u32 = row.get(1)?;
            let layer_id: i64 = row.get(2)?;
            let geometry_data: Vec<u8> = row.get(3)?;
            let properties_data: String = row.get(4)?;
            let visible: i32 = row.get(5)?;
            let locked: i32 = row.get(6)?;

            Ok((
                id,
                generation,
                layer_id,
                geometry_data,
                properties_data,
                visible,
                locked,
            ))
        })?
        .filter_map(|r| r.ok())
        .filter_map(
            |(id, generation, layer_id, geometry_data, properties_data, visible, locked)| {
                let geometry: Geometry = serde_json::from_slice(&geometry_data).ok()?;
                let properties: Properties = serde_json::from_str(&properties_data).ok()?;

                Some(Entity {
                    id: EntityId::from_raw(id as u64, generation),
                    geometry,
                    properties,
                    layer_id: EntityId::from_raw(layer_id as u64, 0),
                    visible: visible != 0,
                    locked: locked != 0,
                })
            },
        )
        .collect();

    for entity in entities {
        document.entities_mut().insert(entity.id, entity);
    }

    // 加载视图
    let mut stmt = conn.prepare("SELECT name, center_x, center_y, zoom FROM views")?;
    document.views = stmt
        .query_map([], |row| {
            Ok(SavedView {
                name: row.get(0)?,
                center_x: row.get(1)?,
                center_y: row.get(2)?,
                zoom: row.get(3)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    // 重建空间索引
    document.rebuild_spatial_index();

    Ok(document)
}

#[cfg(test)]
mod tests {
    use super::*;
    use zcad_core::geometry::{Geometry, Line};
    use zcad_core::math::Point2;

    #[test]
    fn test_save_load_roundtrip() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_document.zcad");

        // 创建文档
        let mut doc = Document::new();
        doc.metadata.title = "Test Document".to_string();

        let line = Line::new(Point2::new(0.0, 0.0), Point2::new(100.0, 100.0));
        let entity = Entity::new(Geometry::Line(line));
        doc.add_entity(entity);

        // 保存
        save(&doc, &file_path).expect("Failed to save");

        // 加载
        let loaded = load(&file_path).expect("Failed to load");

        assert_eq!(loaded.metadata.title, "Test Document");
        assert_eq!(loaded.entity_count(), 1);

        // 清理
        std::fs::remove_file(&file_path).ok();
    }
}

