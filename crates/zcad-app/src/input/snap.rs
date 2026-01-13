//! 捕捉和正交约束处理

use zcad_core::entity::Entity;
use zcad_core::math::Point2;
use zcad_core::snap::SnapPoint;
use zcad_ui::state::{DrawingTool, EditState, UiState};

/// 更新捕捉点（优化版：只处理传入的附近实体）
pub fn update_snap<'a>(
    ui_state: &mut UiState,
    entities: impl Iterator<Item = &'a Entity>,
    camera_zoom: f64,
) {
    // 如果捕捉未启用，直接返回
    if !ui_state.snap_state.enabled {
        ui_state.snap_state.current_snap = None;
        return;
    }
    
    let entities: Vec<&Entity> = entities.collect();

    // 获取参考点（绘图状态下的起始点）
    let reference_point = match &ui_state.edit_state {
        EditState::Drawing { points, .. } if !points.is_empty() => Some(points[0]),
        _ => None,
    };

    // 查找捕捉点
    let mut snap = ui_state.snap_state.engine_mut().find_snap_point(
        ui_state.mouse_world_pos,
        &entities,
        camera_zoom,
        reference_point,
    );

    // 特殊处理：绘制多段线时，检查是否接近起点（用于闭合）
    if let EditState::Drawing { tool: DrawingTool::Polyline, points, .. } = &ui_state.edit_state {
        if points.len() >= 2 {
            let start_point = points[0];
            let world_tolerance = ui_state.snap_state.config().tolerance / camera_zoom;
            let dist_to_start = (ui_state.mouse_world_pos - start_point).norm();
            
            if dist_to_start <= world_tolerance {
                // 比当前捕捉点更近，或者没有当前捕捉点
                let should_use_start = match &snap {
                    Some(existing) => dist_to_start < existing.distance,
                    None => true,
                };
                
                if should_use_start {
                    snap = Some(SnapPoint::new(
                        start_point,
                        zcad_core::snap::SnapType::Endpoint,
                        None,
                        dist_to_start,
                    ));
                }
            }
        }
    }

    // 同样处理圆弧：可以捕捉到第一个点
    if let EditState::Drawing { tool: DrawingTool::Arc, points, .. } = &ui_state.edit_state {
        if !points.is_empty() {
            let first_point = points[0];
            let world_tolerance = ui_state.snap_state.config().tolerance / camera_zoom;
            let dist_to_first = (ui_state.mouse_world_pos - first_point).norm();
            
            if dist_to_first <= world_tolerance {
                let should_use_first = match &snap {
                    Some(existing) => dist_to_first < existing.distance,
                    None => true,
                };
                
                if should_use_first {
                    snap = Some(SnapPoint::new(
                        first_point,
                        zcad_core::snap::SnapType::Endpoint,
                        None,
                        dist_to_first,
                    ));
                }
            }
        }
    }

    ui_state.snap_state.current_snap = snap;
}

/// 应用正交约束
/// 
/// 将目标点约束到从参考点出发的水平或垂直方向
pub fn apply_ortho_constraint(ortho_mode: bool, reference: Point2, target: Point2) -> Point2 {
    if !ortho_mode {
        return target;
    }

    let dx = (target.x - reference.x).abs();
    let dy = (target.y - reference.y).abs();

    if dx > dy {
        // 水平方向更近，约束到水平线
        Point2::new(target.x, reference.y)
    } else {
        // 垂直方向更近，约束到垂直线
        Point2::new(reference.x, target.y)
    }
}

/// 获取有效的绘图点（应用捕捉和正交约束）
pub fn get_effective_draw_point(ui_state: &UiState) -> Point2 {
    let base_point = ui_state.effective_point();

    // 如果正在绘图且有参考点，应用正交约束
    if let EditState::Drawing { points, .. } = &ui_state.edit_state {
        if !points.is_empty() && ui_state.ortho_mode {
            let reference = *points.last().unwrap();
            return apply_ortho_constraint(ui_state.ortho_mode, reference, base_point);
        }
    }

    base_point
}
