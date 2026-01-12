//! 鼠标输入处理

use zcad_core::entity::Entity;
use zcad_core::geometry::{Arc, Circle, Dimension, DimensionType, Geometry, Line, Point, Polyline, Text};
use zcad_core::math::Point2;
use zcad_file::Document;
use zcad_ui::state::{DrawingTool, EditState, UiState};

use crate::history_ops::HistoryOperations;
use super::snap::get_effective_draw_point;

/// 处理左键点击
pub fn handle_left_click(
    ui_state: &mut UiState,
    document: &mut Document,
    history: &mut HistoryOperations,
    camera_zoom: f64,
) {
    // 使用捕捉点和正交约束
    let world_pos = get_effective_draw_point(ui_state);

    match &ui_state.edit_state {
        EditState::Idle => match ui_state.current_tool {
            DrawingTool::Line => {
                ui_state.edit_state = EditState::Drawing {
                    tool: DrawingTool::Line,
                    points: vec![world_pos],
                    expected_input: None,
                };
                ui_state.status_message = "指定下一点:".to_string();
            }
            DrawingTool::Circle => {
                ui_state.edit_state = EditState::Drawing {
                    tool: DrawingTool::Circle,
                    points: vec![world_pos],
                    expected_input: None,
                };
                ui_state.status_message = "指定半径:".to_string();
            }
            DrawingTool::Rectangle => {
                ui_state.edit_state = EditState::Drawing {
                    tool: DrawingTool::Rectangle,
                    points: vec![world_pos],
                    expected_input: None,
                };
                ui_state.status_message = "指定对角点:".to_string();
            }
            DrawingTool::Arc => {
                ui_state.edit_state = EditState::Drawing {
                    tool: DrawingTool::Arc,
                    points: vec![world_pos],
                    expected_input: None,
                };
                ui_state.status_message = "圆弧: 指定第二点:".to_string();
            }
            DrawingTool::Polyline => {
                ui_state.edit_state = EditState::Drawing {
                    tool: DrawingTool::Polyline,
                    points: vec![world_pos],
                    expected_input: None,
                };
                ui_state.status_message = "多段线: 指定下一点 (右键结束):".to_string();
            }
            DrawingTool::Point => {
                // 点直接创建，不需要绘图状态
                let point = Point::from_point2(world_pos);
                let entity = Entity::new(Geometry::Point(point));
                history.add_entity_with_history(document, entity, "创建点");
                ui_state.status_message = "点已创建".to_string();
            }
            DrawingTool::Select => {
                let hits = document.query_point(&world_pos, 5.0 / camera_zoom);
                ui_state.clear_selection();
                if let Some(entity) = hits.first() {
                    ui_state.add_to_selection(entity.id);
                    ui_state.status_message = format!("已选择: {}", entity.geometry().map(|g| g.type_name()).unwrap_or("Unknown"));
                } else {
                    ui_state.status_message.clear();
                }
            }
            DrawingTool::None => {}
            DrawingTool::Text => {
                // 进入文本输入状态
                ui_state.edit_state = EditState::TextInput {
                    position: world_pos,
                    content: String::new(),
                    height: 2.5, // 默认文本高度
                };
                ui_state.status_message = "输入文本内容 (在命令行中输入，回车确认):".to_string();
                ui_state.should_focus_command_line = true;
            }
            DrawingTool::Table => {
                // 表格工具：进入绘制状态，等待指定对角点
                ui_state.edit_state = EditState::Drawing {
                    tool: DrawingTool::Table,
                    points: vec![world_pos],
                    expected_input: None,
                };
                ui_state.status_message = "表格: 指定对角点 (输入 3x4 设置行列数):".to_string();
            }
            DrawingTool::Dimension => {
                ui_state.edit_state = EditState::Drawing {
                    tool: DrawingTool::Dimension,
                    points: vec![world_pos],
                    expected_input: None,
                };
                ui_state.status_message = "标注: 指定第二点:".to_string();
            }
            DrawingTool::DimensionRadius => {
                // 查找点击位置的圆或圆弧
                let hits = document.query_point(&world_pos, 10.0 / camera_zoom);
                if let Some(entity) = hits.iter().find(|e| {
                    e.geometry().map(|g| matches!(g, Geometry::Circle(_) | Geometry::Arc(_))).unwrap_or(false)
                }) {
                    // 找到圆或圆弧，开始标注
                    let Some(geometry) = entity.geometry() else { return; };
                    let (center, radius) = match geometry {
                        Geometry::Circle(c) => (c.center, c.radius),
                        Geometry::Arc(a) => (a.center, a.radius),
                        _ => return,
                    };
                    ui_state.edit_state = EditState::Drawing {
                        tool: DrawingTool::DimensionRadius,
                        points: vec![center, Point2::new(center.x + radius, center.y)],
                        expected_input: None,
                    };
                    ui_state.status_message = "半径标注: 指定文字位置:".to_string();
                } else {
                    ui_state.status_message = "请选择圆或圆弧".to_string();
                }
            }
            DrawingTool::DimensionDiameter => {
                // 查找点击位置的圆或圆弧
                let hits = document.query_point(&world_pos, 10.0 / camera_zoom);
                if let Some(entity) = hits.iter().find(|e| {
                    e.geometry().map(|g| matches!(g, Geometry::Circle(_) | Geometry::Arc(_))).unwrap_or(false)
                }) {
                    let Some(geometry) = entity.geometry() else { return; };
                    let (center, radius) = match geometry {
                        Geometry::Circle(c) => (c.center, c.radius),
                        Geometry::Arc(a) => (a.center, a.radius),
                        _ => return,
                    };
                    ui_state.edit_state = EditState::Drawing {
                        tool: DrawingTool::DimensionDiameter,
                        points: vec![center, Point2::new(center.x + radius, center.y)],
                        expected_input: None,
                    };
                    ui_state.status_message = "直径标注: 指定文字位置:".to_string();
                } else {
                    ui_state.status_message = "请选择圆或圆弧".to_string();
                }
            }
        },
        EditState::Drawing { tool, points, .. } => {
            let tool = *tool;
            let mut new_points = points.clone();
            new_points.push(world_pos);

            match tool {
                DrawingTool::Line => {
                    if new_points.len() >= 2 {
                        let line = Line::new(new_points[0], new_points[1]);
                        let entity = Entity::new(Geometry::Line(line));
                        history.add_entity_with_history(document, entity, "创建直线");
                        ui_state.edit_state = EditState::Drawing {
                            tool: DrawingTool::Line,
                            points: vec![new_points[1]],
                            expected_input: None,
                        };
                        ui_state.status_message = "直线已创建。下一点:".to_string();
                    }
                }
                DrawingTool::Circle => {
                    if new_points.len() >= 2 {
                        let radius = (new_points[1] - new_points[0]).norm();
                        let circle = Circle::new(new_points[0], radius);
                        let entity = Entity::new(Geometry::Circle(circle));
                        history.add_entity_with_history(document, entity, "创建圆");
                        ui_state.edit_state = EditState::Idle;
                        ui_state.status_message = "圆已创建".to_string();
                    }
                }
                DrawingTool::Rectangle => {
                    if new_points.len() >= 2 {
                        let p1 = new_points[0];
                        let p2 = new_points[1];
                        let rect = Polyline::from_points(
                            [
                                Point2::new(p1.x, p1.y),
                                Point2::new(p2.x, p1.y),
                                Point2::new(p2.x, p2.y),
                                Point2::new(p1.x, p2.y),
                            ],
                            true,
                        );
                        let entity = Entity::new(Geometry::Polyline(rect));
                        history.add_entity_with_history(document, entity, "创建矩形");
                        ui_state.edit_state = EditState::Idle;
                        ui_state.status_message = "矩形已创建".to_string();
                    }
                }
                DrawingTool::Arc => {
                    // 三点圆弧：起点、经过点、终点
                    if new_points.len() == 2 {
                        // 第二个点
                        ui_state.edit_state = EditState::Drawing {
                            tool: DrawingTool::Arc,
                            points: new_points,
                            expected_input: None,
                        };
                        ui_state.status_message = "圆弧: 指定终点:".to_string();
                    } else if new_points.len() >= 3 {
                        // 三个点，创建圆弧
                        if let Some(arc) = Arc::from_three_points(
                            new_points[0],
                            new_points[1],
                            new_points[2],
                        ) {
                            let entity = Entity::new(Geometry::Arc(arc));
                            history.add_entity_with_history(document, entity, "创建圆弧");
                            ui_state.status_message = "圆弧已创建".to_string();
                        } else {
                            ui_state.status_message = "无法创建圆弧（三点共线）".to_string();
                        }
                        ui_state.edit_state = EditState::Idle;
                    }
                }
                DrawingTool::Polyline => {
                    // 检查是否点击了起点（闭合多段线）
                    if new_points.len() >= 3 {
                        let start = new_points[0];
                        let current = new_points[new_points.len() - 1];
                        let tolerance = 0.001; // 很小的容差，因为捕捉已经对齐了
                        
                        if (current - start).norm() < tolerance {
                            // 点击了起点，创建闭合多段线
                            new_points.pop(); // 移除重复的终点
                            let polyline = Polyline::from_points(new_points, true); // closed = true
                            let entity = Entity::new(Geometry::Polyline(polyline));
                            history.add_entity_with_history(document, entity, "创建闭合多段线");
                            ui_state.edit_state = EditState::Idle;
                            ui_state.status_message = "闭合多段线已创建".to_string();
                            return;
                        }
                    }
                    
                    // 否则继续添加点
                    ui_state.edit_state = EditState::Drawing {
                        tool: DrawingTool::Polyline,
                        points: new_points,
                        expected_input: None,
                    };
                    ui_state.status_message = "多段线: 指定下一点 (右键结束, 点击起点闭合):".to_string();
                }
                DrawingTool::Dimension => {
                    if new_points.len() == 2 {
                        // 第二点已指定，等待第三点（标注线位置）
                        ui_state.edit_state = EditState::Drawing {
                            tool: DrawingTool::Dimension,
                            points: new_points,
                            expected_input: None,
                        };
                        ui_state.status_message = "标注: 指定标注线位置:".to_string();
                    } else if new_points.len() == 3 {
                        // 第三点已指定，创建标注
                        let dim = Dimension::new(new_points[0], new_points[1], new_points[2]);
                        let entity = Entity::new(Geometry::Dimension(dim));
                        history.add_entity_with_history(document, entity, "创建标注");
                        ui_state.edit_state = EditState::Idle;
                        ui_state.status_message = "标注已创建".to_string();
                    }
                }
                DrawingTool::DimensionRadius => {
                    // points[0] = 圆心, points[1] = 圆上点, points[2] = 文字位置
                    if new_points.len() >= 3 {
                        let center = new_points[0];
                        let radius_point = new_points[1];
                        let text_pos = new_points[2];
                        
                        let mut dim = Dimension::new(center, radius_point, text_pos);
                        dim.dim_type = DimensionType::Radius;
                        let entity = Entity::new(Geometry::Dimension(dim));
                        history.add_entity_with_history(document, entity, "创建半径标注");
                        ui_state.edit_state = EditState::Idle;
                        ui_state.status_message = "半径标注已创建".to_string();
                    }
                }
                DrawingTool::DimensionDiameter => {
                    // points[0] = 圆心, points[1] = 圆上点, points[2] = 文字位置
                    if new_points.len() >= 3 {
                        let center = new_points[0];
                        let p_rad = new_points[1];
                        let text_pos = new_points[2];
                        
                        let radius = (p_rad - center).norm();
                        let dir = if (text_pos - center).norm() > 0.001 {
                            (text_pos - center).normalize()
                        } else {
                            zcad_core::math::Vector2::x()
                        };
                        
                        let p1 = center - dir * radius;
                        let p2 = center + dir * radius;
                        
                        let mut dim = Dimension::new(p1, p2, text_pos);
                        dim.dim_type = DimensionType::Diameter;
                        let entity = Entity::new(Geometry::Dimension(dim));
                        history.add_entity_with_history(document, entity, "创建直径标注");
                        ui_state.edit_state = EditState::Idle;
                        ui_state.status_message = "直径标注已创建".to_string();
                    }
                }
                DrawingTool::Table => {
                    // 两点确定表格大小
                    if new_points.len() >= 2 {
                        let p1 = new_points[0];
                        let p2 = new_points[1];
                        
                        let width = (p2.x - p1.x).abs();
                        let height = (p2.y - p1.y).abs();
                        
                        if width > 0.1 && height > 0.1 {
                            // 确定左上角
                            let top_left = Point2::new(
                                p1.x.min(p2.x),
                                p1.y.max(p2.y),
                            );
                            
                            // 默认 3x3 表格
                            let rows = 3;
                            let cols = 3;
                            
                            let mut table = zcad_core::geometry::Table::new(top_left, rows, cols);
                            
                            // 设置列宽和行高
                            let col_width = width / cols as f64;
                            let row_height = height / rows as f64;
                            
                            for i in 0..cols {
                                table.set_column_width(i, col_width);
                            }
                            for i in 0..rows {
                                table.set_row_height(i, row_height);
                            }
                            
                            // 调整文本高度
                            table.style.text_height = (row_height * 0.5).min(2.5);
                            
                            let entity = Entity::new(Geometry::Table(table));
                            history.add_entity_with_history(document, entity, "创建表格");
                            ui_state.edit_state = EditState::Idle;
                            ui_state.status_message = "表格已创建".to_string();
                        }
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
}

/// 处理右键点击（结束多段线等）
pub fn handle_right_click(
    ui_state: &mut UiState,
    document: &mut Document,
    history: &mut HistoryOperations,
) {
    // 先提取需要的信息，避免借用冲突
    let (is_polyline, points_to_create) = if let EditState::Drawing { tool, points, .. } = &ui_state.edit_state {
        if *tool == DrawingTool::Polyline && points.len() >= 2 {
            (true, Some(points.clone()))
        } else if *tool == DrawingTool::Polyline {
            (true, None) // 点数不够
        } else {
            (false, None)
        }
    } else {
        (false, None)
    };

    if is_polyline {
        if let Some(pts) = points_to_create {
            let polyline = Polyline::from_points(pts.clone(), false);
            let entity = Entity::new(Geometry::Polyline(polyline));
            history.add_entity_with_history(document, entity, "创建多段线");
            ui_state.status_message = format!("多段线已创建 ({} 个点)", pts.len());
        } else {
            ui_state.status_message = "取消".to_string();
        }
        ui_state.edit_state = EditState::Idle;
    } else {
        ui_state.cancel();
    }
}
