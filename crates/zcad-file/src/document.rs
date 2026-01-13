//! CAD文档数据模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use zcad_core::entity::{Entity, EntityId};
use zcad_core::layer::LayerManager;
use zcad_core::layout::LayoutManager;
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

/// 坐标变换信息（用于大坐标归一化后的显示）
#[derive(Debug, Clone, Copy, Default)]
pub struct CoordinateTransform {
    /// 偏移量（内部坐标 + offset = 显示坐标）
    pub offset: zcad_core::math::Vector2,
    /// 缩放因子（内部坐标 * scale = 显示坐标）
    pub scale: f64,
}

impl CoordinateTransform {
    /// 创建单位变换
    pub fn identity() -> Self {
        Self {
            offset: zcad_core::math::Vector2::new(0.0, 0.0),
            scale: 1.0,
        }
    }

    /// 将内部坐标转换为显示坐标（给用户看的）
    pub fn to_display(&self, internal: zcad_core::math::Point2) -> zcad_core::math::Point2 {
        zcad_core::math::Point2::new(
            internal.x / self.scale + self.offset.x,
            internal.y / self.scale + self.offset.y,
        )
    }

    /// 将显示坐标转换为内部坐标（用户输入的）
    pub fn to_internal(&self, display: zcad_core::math::Point2) -> zcad_core::math::Point2 {
        zcad_core::math::Point2::new(
            (display.x - self.offset.x) * self.scale,
            (display.y - self.offset.y) * self.scale,
        )
    }

    /// 将内部距离转换为显示距离
    pub fn distance_to_display(&self, internal_distance: f64) -> f64 {
        internal_distance / self.scale
    }

    /// 是否有变换（非单位变换）
    pub fn has_transform(&self) -> bool {
        self.offset.x.abs() > 0.001 || self.offset.y.abs() > 0.001 || (self.scale - 1.0).abs() > 0.0001
    }
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

    /// 布局管理器
    pub layout_manager: LayoutManager,

    /// 是否已修改
    modified: bool,

    /// 文件路径（如果已保存）
    file_path: Option<std::path::PathBuf>,

    /// 坐标变换（归一化偏移量）
    pub coordinate_transform: CoordinateTransform,
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
            layout_manager: LayoutManager::new(),
            modified: false,
            file_path: None,
            coordinate_transform: CoordinateTransform::identity(),
        }
    }

    /// 从文件加载
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Self, crate::FileError> {
        let path = path.as_ref();

        match path.extension().and_then(|e| e.to_str()).map(|s| s.to_lowercase()).as_deref() {
            Some("zcad") => crate::native::load(path),
            Some("dxf") => crate::dxf_io::import(path),
            #[cfg(feature = "dwg")]
            Some("dwg") => crate::dwg_io::import(path),
            #[cfg(not(feature = "dwg"))]
            Some("dwg") => Err(crate::FileError::InvalidFormat(
                "DWG support not enabled. Rebuild with 'dwg' feature.".to_string(),
            )),
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
            .filter(|e| e.geometry().map(|g| g.contains_point(point, tolerance)).unwrap_or(false))
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

    /// 归一化坐标（用于建筑图等大坐标场景）
    /// 
    /// 检测坐标范围，如果坐标值很大：
    /// 1. 将图纸中心平移到原点
    /// 2. 可选：将毫米转换为米（除以1000）
    /// 
    /// 返回 (是否进行了归一化, 平移偏移量, 缩放因子)
    pub fn normalize_coordinates(&mut self, convert_mm_to_m: bool) -> (bool, zcad_core::math::Vector2, f64) {
        use zcad_core::math::Vector2;

        let bounds = match self.bounds() {
            Some(b) => b,
            None => return (false, Vector2::new(0.0, 0.0), 1.0),
        };

        let center = bounds.center();
        let max_coord = center.x.abs().max(center.y.abs());
        
        // 阈值：如果最大坐标超过 100,000（典型建筑图坐标），则归一化
        let threshold = 100_000.0;
        
        if max_coord < threshold {
            return (false, Vector2::new(0.0, 0.0), 1.0);
        }

        // 保存原始中心点（用于显示坐标转换）
        let original_center = center;
        
        // 计算平移偏移量（将中心移到原点）
        let offset = Vector2::new(-center.x, -center.y);
        
        // 计算缩放因子
        let scale_factor = if convert_mm_to_m { 0.001 } else { 1.0 };

        // 对所有实体应用变换
        for entity in self.entities.values_mut() {
            if let Some(geometry) = entity.geometry_mut() {
                // 先平移
                geometry.translate(offset);
                // 再缩放
                if convert_mm_to_m {
                    geometry.scale(scale_factor);
                }
            }
        }

        // 保存坐标变换信息（用于显示原始坐标给用户）
        // 内部坐标 * (1/scale) + original_center = 原始坐标
        self.coordinate_transform = CoordinateTransform {
            offset: Vector2::new(original_center.x, original_center.y),
            scale: scale_factor,
        };

        // 更新单位（内部单位）
        if convert_mm_to_m {
            self.metadata.units = "m".to_string();
        }

        // 重建空间索引
        self.rebuild_spatial_index();

        tracing::info!(
            "Coordinates normalized: offset=({:.2}, {:.2}), scale={}, display_offset=({:.2}, {:.2})",
            offset.x, offset.y, scale_factor, 
            self.coordinate_transform.offset.x, self.coordinate_transform.offset.y
        );

        (true, offset, scale_factor)
    }

    /// 检查是否需要归一化（坐标值是否很大）
    pub fn needs_normalization(&self) -> bool {
        if let Some(bounds) = self.bounds() {
            let center = bounds.center();
            let max_coord = center.x.abs().max(center.y.abs());
            max_coord >= 100_000.0
        } else {
            false
        }
    }

    /// 获取建议的归一化参数
    pub fn get_normalization_info(&self) -> Option<(zcad_core::math::Point2, f64)> {
        let bounds = self.bounds()?;
        let center = bounds.center();
        let width = bounds.max.x - bounds.min.x;
        let height = bounds.max.y - bounds.min.y;
        let max_size = width.max(height);
        
        // 如果尺寸很大（>10000），建议转换单位
        let suggested_scale = if max_size > 10_000.0 { 0.001 } else { 1.0 };
        
        Some((center, suggested_scale))
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

