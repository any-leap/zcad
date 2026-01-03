//! CAD文档数据模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use zcad_core::entity::{Entity, EntityId};
use zcad_core::layer::LayerManager;
use zcad_core::math::BoundingBox2;
use zcad_core::spatial::SpatialIndex;

/// 文档元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    /// 文档唯一标识
    pub id: Uuid,

    /// 文档标题
    pub title: String,

    /// 作者
    pub author: String,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 最后修改时间
    pub modified_at: DateTime<Utc>,

    /// 文件格式版本
    pub format_version: u32,

    /// 单位（mm, cm, m, inch, feet）
    pub units: String,

    /// 自定义属性
    pub custom_properties: HashMap<String, String>,
}

impl Default for DocumentMetadata {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            title: "Untitled".to_string(),
            author: String::new(),
            created_at: Utc::now(),
            modified_at: Utc::now(),
            format_version: 1,
            units: "mm".to_string(),
            custom_properties: HashMap::new(),
        }
    }
}

/// 保存的视图
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedView {
    pub name: String,
    pub center_x: f64,
    pub center_y: f64,
    pub zoom: f64,
}

/// CAD文档
#[derive(Debug)]
pub struct Document {
    /// 元数据
    pub metadata: DocumentMetadata,

    /// 所有实体
    entities: HashMap<EntityId, Entity>,

    /// 图层管理器
    pub layers: LayerManager,

    /// 空间索引
    spatial_index: SpatialIndex,

    /// 保存的视图
    pub views: Vec<SavedView>,

    /// 是否已修改
    modified: bool,

    /// 文件路径（如果已保存）
    file_path: Option<std::path::PathBuf>,
}

impl Document {
    /// 创建新文档
    pub fn new() -> Self {
        Self {
            metadata: DocumentMetadata::default(),
            entities: HashMap::new(),
            layers: LayerManager::new(),
            spatial_index: SpatialIndex::default_grid(),
            views: Vec::new(),
            modified: false,
            file_path: None,
        }
    }

    /// 从文件加载
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Self, crate::FileError> {
        let path = path.as_ref();

        match path.extension().and_then(|e| e.to_str()) {
            Some("zcad") => crate::native::load(path),
            Some("dxf") => crate::dxf_io::import(path),
            _ => Err(crate::FileError::InvalidFormat(
                "Unknown file extension".to_string(),
            )),
        }
    }

    /// 保存文件
    pub fn save(&mut self) -> Result<(), crate::FileError> {
        if let Some(path) = &self.file_path.clone() {
            self.save_as(path)
        } else {
            Err(crate::FileError::InvalidFormat(
                "No file path set".to_string(),
            ))
        }
    }

    /// 另存为
    pub fn save_as(&mut self, path: impl AsRef<std::path::Path>) -> Result<(), crate::FileError> {
        let path = path.as_ref();

        match path.extension().and_then(|e| e.to_str()) {
            Some("zcad") => crate::native::save(self, path)?,
            Some("dxf") => crate::dxf_io::export(self, path)?,
            _ => {
                return Err(crate::FileError::InvalidFormat(
                    "Unknown file extension".to_string(),
                ))
            }
        }

        self.file_path = Some(path.to_path_buf());
        self.modified = false;
        self.metadata.modified_at = Utc::now();

        Ok(())
    }

    /// 添加实体
    pub fn add_entity(&mut self, entity: Entity) -> EntityId {
        let id = entity.id;
        let bbox = entity.bounding_box();

        self.spatial_index.insert(id, bbox);
        self.entities.insert(id, entity);
        self.modified = true;

        id
    }

    /// 删除实体
    pub fn remove_entity(&mut self, id: &EntityId) -> Option<Entity> {
        self.spatial_index.remove(id);
        self.modified = true;
        self.entities.remove(id)
    }

    /// 获取实体
    pub fn get_entity(&self, id: &EntityId) -> Option<&Entity> {
        self.entities.get(id)
    }

    /// 获取可变实体
    pub fn get_entity_mut(&mut self, id: &EntityId) -> Option<&mut Entity> {
        self.modified = true;
        self.entities.get_mut(id)
    }

    /// 更新实体（并更新空间索引）
    pub fn update_entity(&mut self, id: &EntityId, entity: Entity) {
        let bbox = entity.bounding_box();
        self.spatial_index.update(*id, bbox);
        self.entities.insert(*id, entity);
        self.modified = true;
    }

    /// 查询矩形区域内的实体
    pub fn query_rect(&self, rect: &BoundingBox2) -> Vec<&Entity> {
        self.spatial_index
            .query_rect(rect)
            .iter()
            .filter_map(|id| self.entities.get(id))
            .collect()
    }

    /// 查询点附近的实体
    pub fn query_point(&self, point: &zcad_core::math::Point2, tolerance: f64) -> Vec<&Entity> {
        let rect = BoundingBox2::new(
            zcad_core::math::Point2::new(point.x - tolerance, point.y - tolerance),
            zcad_core::math::Point2::new(point.x + tolerance, point.y + tolerance),
        );

        self.query_rect(&rect)
            .into_iter()
            .filter(|e| e.geometry.contains_point(point, tolerance))
            .collect()
    }

    /// 获取所有实体
    pub fn all_entities(&self) -> impl Iterator<Item = &Entity> {
        self.entities.values()
    }

    /// 获取实体数量
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// 计算所有实体的包围盒
    pub fn bounds(&self) -> Option<BoundingBox2> {
        let mut iter = self.entities.values();
        let first = iter.next()?;
        let mut bbox = first.bounding_box();

        for entity in iter {
            bbox = bbox.union(&entity.bounding_box());
        }

        Some(bbox)
    }

    /// 是否已修改
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    /// 标记为已保存
    pub fn mark_saved(&mut self) {
        self.modified = false;
    }

    /// 获取文件路径
    pub fn file_path(&self) -> Option<&std::path::Path> {
        self.file_path.as_deref()
    }

    /// 设置文件路径
    pub fn set_file_path(&mut self, path: impl AsRef<std::path::Path>) {
        self.file_path = Some(path.as_ref().to_path_buf());
    }

    /// 获取实体的可变HashMap引用（用于文件加载）
    pub(crate) fn entities_mut(&mut self) -> &mut HashMap<EntityId, Entity> {
        &mut self.entities
    }

    /// 重建空间索引
    pub fn rebuild_spatial_index(&mut self) {
        self.spatial_index.clear();
        for (id, entity) in &self.entities {
            self.spatial_index.insert(*id, entity.bounding_box());
        }
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

