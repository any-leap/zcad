//! ZCAD 核心应用程序

use eframe::egui;
use tracing::info;

use zcad_core::entity::Entity;
use zcad_core::geometry::{Circle, Geometry, Line, Polyline, Text};
use zcad_core::math::Point2;
use zcad_core::properties::Color;
use zcad_file::Document;
use zcad_ui::state::{Command, EditState, UiState};

use crate::camera::Camera;
use crate::file_ops::FileOperations;
use crate::history_ops::HistoryOperations;
use crate::input::{handle_left_click, handle_right_click, update_snap, get_effective_draw_point};
use crate::input::handle_keyboard_shortcuts;
use crate::rendering::{self, RenderContext};
use crate::theme::THEME;
use crate::ui::{self, LayerInfo, SelectedEntityInfo, extract_geometry_properties};
use crate::ui_state::UiStateManager;

/// ZCAD 应用程序
pub struct ZcadApp {
    pub document: Document,
    pub ui_state: UiState,
    pub camera: Camera,
    pub file_ops: FileOperations,
    pub history: HistoryOperations,
}

impl Default for ZcadApp {
    fn default() -> Self {
        let mut app = Self {
            document: Document::new(),
            ui_state: UiState::default(),
            camera: Camera::default(),
            file_ops: FileOperations::new(),
            history: HistoryOperations::new(),
        };
        app.create_demo_content();
        app
    }
}

// 实现 UiStateManager trait for UiState
impl UiStateManager for UiState {
    fn clear_selection(&mut self) {
        self.selected_entities.clear();
    }
    
    fn set_status_message(&mut self, message: String) {
        self.status_message = message;
    }
    
    fn add_to_selection(&mut self, id: zcad_core::entity::EntityId) {
        if !self.selected_entities.contains(&id) {
            self.selected_entities.push(id);
        }
    }
}

impl ZcadApp {
    /// 创建示例内容
    fn create_demo_content(&mut self) {
        // 创建示例线条
        for i in 0..10 {
            let x = i as f64 * 50.0;
            let line = Line::new(Point2::new(x, 0.0), Point2::new(x, 200.0));
            let mut entity = Entity::new(Geometry::Line(line));
            entity.visual_properties.color = Color::CYAN;
            self.document.add_entity(entity);
        }

        // 创建圆
        let circle = Circle::new(Point2::new(250.0, 100.0), 80.0);
        let mut entity = Entity::new(Geometry::Circle(circle));
        entity.visual_properties.color = Color::YELLOW;
        self.document.add_entity(entity);

        // 创建矩形
        let rect = Polyline::from_points(
            [
                Point2::new(400.0, 50.0),
                Point2::new(550.0, 50.0),
                Point2::new(550.0, 150.0),
                Point2::new(400.0, 150.0),
            ],
            true,
        );
        let mut entity = Entity::new(Geometry::Polyline(rect));
        entity.visual_properties.color = Color::GREEN;
        self.document.add_entity(entity);

        info!("Created {} demo entities", self.document.entity_count());
    }

    /// 删除选中的实体
    fn delete_selected(&mut self) {
        let ids = self.ui_state.selected_entities.clone();
        self.history.delete_selected_entities(&mut self.document, &mut self.ui_state, &ids);
    }

    /// 执行撤销
    fn do_undo(&mut self) {
        self.history.do_undo(&mut self.document, &mut self.ui_state);
    }

    /// 执行重做
    fn do_redo(&mut self) {
        self.history.do_redo(&mut self.document, &mut self.ui_state);
    }

    /// 缩放到适合
    fn zoom_to_fit(&mut self) {
        self.camera.zoom_to_fit(self.document.bounds().as_ref());
    }

    /// 处理菜单结果
    fn handle_menu_result(&mut self, result: ui::menu::MenuResult) {
        if result.new_document {
            self.document = Document::new();
            self.ui_state.clear_selection();
            self.ui_state.status_message = "新文档".to_string();
        }
        if result.open_dialog {
            self.file_ops.show_open_dialog();
        }
        if result.save {
            self.file_ops.quick_save(&mut self.document, &mut self.ui_state);
        }
        if result.save_as {
            self.file_ops.show_save_dialog(self.document.file_path());
        }
        if result.exit {
            std::process::exit(0);
        }
        if result.delete {
            self.delete_selected();
        }
        if result.undo {
            self.do_undo();
        }
        if result.redo {
            self.do_redo();
        }
        if result.zoom_fit {
            self.zoom_to_fit();
        }
        if result.toggle_grid {
            self.ui_state.show_grid = !self.ui_state.show_grid;
        }
        if result.toggle_ortho {
            self.ui_state.ortho_mode = !self.ui_state.ortho_mode;
        }
        if let Some(tool) = result.set_tool {
            self.ui_state.set_tool(tool);
        }
    }

    /// 处理工具栏结果
    fn handle_toolbar_result(&mut self, result: ui::toolbar::ToolbarResult) {
        if let Some(tool) = result.set_tool {
            self.ui_state.set_tool(tool);
        }
        if result.delete {
            self.delete_selected();
        }
        if result.undo {
            self.do_undo();
        }
        if result.redo {
            self.do_redo();
        }
        if result.toggle_ortho {
            self.ui_state.ortho_mode = !self.ui_state.ortho_mode;
        }
        if result.toggle_grid {
            self.ui_state.show_grid = !self.ui_state.show_grid;
        }
        if result.zoom_fit {
            self.zoom_to_fit();
        }
    }

    /// 处理键盘结果
    fn handle_keyboard_result(&mut self, result: crate::input::keyboard::KeyboardResult) {
        if result.should_new_document {
            self.document = Document::new();
            self.ui_state.clear_selection();
            self.ui_state.status_message = "新文档".to_string();
        }
        if result.should_open_dialog {
            self.file_ops.show_open_dialog();
        }
        if result.should_save {
            self.file_ops.quick_save(&mut self.document, &mut self.ui_state);
        }
        if result.should_save_as {
            self.file_ops.show_save_dialog(self.document.file_path());
        }
        if result.should_delete {
            self.delete_selected();
        }
        if result.should_undo {
            self.do_undo();
        }
        if result.should_redo {
            self.do_redo();
        }
        if result.should_zoom_fit {
            self.zoom_to_fit();
        }
    }

    /// 准备图层信息
    fn prepare_layers_info(&self) -> Vec<LayerInfo> {
        self.document.layers.all_layers().iter()
            .map(|l| LayerInfo {
                name: l.name.clone(),
                color: (l.color.r, l.color.g, l.color.b),
                is_current: l.name == self.document.layers.current_layer().name,
            })
            .collect()
    }

    /// 准备选中实体信息
    fn prepare_selected_info(&self) -> Option<SelectedEntityInfo> {
        if self.ui_state.selected_entities.len() == 1 {
            self.document.get_entity(&self.ui_state.selected_entities[0]).and_then(|e| {
                e.geometry().map(|g| SelectedEntityInfo {
                    type_name: g.type_name().to_string(),
                    properties: extract_geometry_properties(g),
                })
            })
        } else {
            None
        }
    }

    /// 处理命令
    fn handle_command(&mut self, cmd: Command) {
        match cmd {
            Command::SetTool(tool) => {
                self.ui_state.set_tool(tool);
            }
            Command::DeleteSelected => {
                self.delete_selected();
            }
            Command::Undo => {
                self.do_undo();
            }
            Command::Redo => {
                self.do_redo();
            }
            Command::ZoomExtents => {
                self.zoom_to_fit();
            }
            Command::New => {
                self.document = Document::new();
                self.ui_state.clear_selection();
                self.ui_state.status_message = "新文档".to_string();
            }
            Command::Open => {
                self.file_ops.show_open_dialog();
            }
            Command::Save => {
                self.file_ops.quick_save(&mut self.document, &mut self.ui_state);
            }
            Command::CreateText { position, content, height } => {
                let text = Text::new(position, content, height);
                let entity = Entity::new(Geometry::Text(text));
                self.history.add_entity_with_history(&mut self.document, entity, "创建文本");
                self.ui_state.status_message = "文本已创建".to_string();
            }
            Command::Move => {
                // TODO: 实现移动命令
                self.ui_state.status_message = "移动命令".to_string();
            }
            Command::Copy => {
                // TODO: 实现复制命令
                self.ui_state.status_message = "复制命令".to_string();
            }
            Command::Rotate => {
                // TODO: 实现旋转命令
                self.ui_state.status_message = "旋转命令".to_string();
            }
            Command::Scale => {
                // TODO: 实现缩放命令
                self.ui_state.status_message = "缩放命令".to_string();
            }
            Command::Mirror => {
                // TODO: 实现镜像命令
                self.ui_state.status_message = "镜像命令".to_string();
            }
            Command::ExportDxf => {
                // TODO: 实现导出 DXF
                self.ui_state.status_message = "导出 DXF".to_string();
            }
            Command::DataInput(data) => {
                // 数据输入在绘图状态下处理
                self.ui_state.status_message = format!("数据输入: {}", data);
            }
        }
    }
}

impl eframe::App for ZcadApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 处理文件操作
        let camera = self.camera.clone();
        self.file_ops.process(&mut self.document, &mut self.ui_state, || {
            // 这个闭包会在文件打开后调用
        });
        
        // 更新窗口标题
        let title = if let Some(path) = self.document.file_path() {
            let modified = if self.document.is_modified() { "*" } else { "" };
            format!("ZCAD - {}{}", path.display(), modified)
        } else {
            let modified = if self.document.is_modified() { "*" } else { "" };
            format!("ZCAD - Untitled{}", modified)
        };
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));
        
        // 应用现代化主题
        THEME.apply(ctx);

        // UI状态快照
        let current_tool = self.ui_state.current_tool;
        let ortho = self.ui_state.ortho_mode;
        let grid = self.ui_state.show_grid;
        let status = self.ui_state.status_message.clone();
        let mouse_world = self.ui_state.mouse_world_pos;
        let entity_count = self.document.entity_count();
        let selected_count = self.ui_state.selected_entities.len();
        let snap_enabled = self.ui_state.snap_state.enabled;
        let snap_info = self.ui_state.snap_state.current_snap.as_ref().map(|s| {
            (s.snap_type.name(), s.point)
        });
        let effective_pos = self.ui_state.effective_point();

        // 准备面板数据
        let layers_info = self.prepare_layers_info();
        let selected_info = self.prepare_selected_info();

        // ===== 显示菜单 =====
        let menu_result = ui::show_menu(ctx, grid, ortho);
        self.handle_menu_result(menu_result);

        // ===== 显示工具栏 =====
        let toolbar_result = ui::show_toolbar(ctx, current_tool, ortho, grid);
        self.handle_toolbar_result(toolbar_result);

        // ===== 显示状态栏和命令行 =====
        let statusbar_result = ui::show_statusbar(
            ctx,
            &status,
            snap_enabled,
            snap_info,
            effective_pos,
            entity_count,
            selected_count,
            &mut self.ui_state.command_input,
            &mut self.ui_state.should_focus_command_line,
        );
        if statusbar_result.toggle_snap {
            self.ui_state.snap_state.enabled = !self.ui_state.snap_state.enabled;
        }
        // 处理命令输入
        if let Some(input) = statusbar_result.command_input {
            if let Some(cmd) = self.ui_state.execute_command(&input) {
                self.handle_command(cmd);
            }
        }

        // ===== 显示图层面板 =====
        ui::show_layers_panel(ctx, &layers_info);

        // ===== 显示属性面板 =====
        ui::show_properties_panel(
            ctx,
            selected_info.as_ref(),
            selected_count,
            current_tool,
            mouse_world,
        );

        // ===== 中央绘图区域 =====
        egui::CentralPanel::default()
            .frame(THEME.canvas_frame())
            .show(ctx, |ui| {
                let available_rect = ui.available_rect_before_wrap();
                self.camera.viewport_size = (available_rect.width(), available_rect.height());
                
                let (response, painter) = ui.allocate_painter(available_rect.size(), egui::Sense::click_and_drag());
                let rect = response.rect;

                // 处理鼠标位置
                if let Some(hover_pos) = response.hover_pos() {
                    self.ui_state.mouse_world_pos = self.camera.screen_to_world(hover_pos, &rect);
                    // 更新捕捉点
                    update_snap(&mut self.ui_state, self.document.all_entities(), self.camera.zoom);
                }

                // 处理滚轮缩放
                let scroll_delta = ui.input(|i| i.raw_scroll_delta);
                if scroll_delta.y.abs() > 0.0 && response.hovered() {
                    if let Some(hover_pos) = response.hover_pos() {
                        self.camera.handle_scroll_zoom(scroll_delta.y, hover_pos, &rect);
                    }
                }

                // 处理中键平移
                if response.dragged_by(egui::PointerButton::Middle) {
                    self.camera.handle_pan(response.drag_delta());
                }

                // 处理左键点击
                if response.clicked_by(egui::PointerButton::Primary) {
                    handle_left_click(
                        &mut self.ui_state,
                        &mut self.document,
                        &mut self.history,
                        self.camera.zoom,
                    );
                }

                // 处理右键（结束多段线或取消）
                if response.clicked_by(egui::PointerButton::Secondary) {
                    handle_right_click(
                        &mut self.ui_state,
                        &mut self.document,
                        &mut self.history,
                    );
                }

                // 处理键盘快捷键
                let keyboard_result = handle_keyboard_shortcuts(ctx, &mut self.ui_state);
                self.handle_keyboard_result(keyboard_result);

                // ===== 绘制 =====
                let render_ctx = RenderContext::new(
                    &painter,
                    &rect,
                    self.camera.center,
                    self.camera.zoom,
                );

                // 绘制网格
                rendering::draw_grid(&render_ctx, self.ui_state.show_grid);

                // 绘制所有实体
                for entity in self.document.all_entities() {
                    let color = if self.ui_state.selected_entities.contains(&entity.id) {
                        Color::from_hex(0x00FF00)
                    } else if entity.visual_properties.color.is_by_layer() {
                        self.document.layers.get_layer_by_id(entity.layer_id)
                            .map(|l| l.color).unwrap_or(Color::WHITE)
                    } else {
                        entity.visual_properties.color
                    };
                    if let Some(geometry) = entity.geometry() {
                        rendering::draw_geometry(&render_ctx, geometry, color);
                    }
                }

                // 绘制预览
                let effective_point = get_effective_draw_point(&self.ui_state);
                rendering::draw_preview(&render_ctx, &self.ui_state.edit_state, effective_point);

                // 绘制正交辅助线
                if self.ui_state.ortho_mode {
                    if let EditState::Drawing { points, .. } = &self.ui_state.edit_state {
                        if let Some(&reference) = points.last() {
                            rendering::draw_ortho_guides(&render_ctx, reference);
                        }
                    }
                }

                // 绘制捕捉标记
                if let Some(ref snap) = self.ui_state.snap_state.current_snap {
                    if self.ui_state.snap_state.enabled {
                        rendering::draw_snap_marker(&render_ctx, snap.snap_type, snap.point);
                    }
                }

                // 绘制十字光标（使用捕捉点如果有的话）
                if response.hovered() {
                    let cursor_pos = self.ui_state.effective_point();
                    rendering::draw_crosshair(&render_ctx, cursor_pos);
                }
            });

        // 请求持续重绘（实现动画效果）
        ctx.request_repaint();
    }
}
