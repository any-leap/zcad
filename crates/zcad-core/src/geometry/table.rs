//! 表格
//!
//! CAD 表格是一种特殊的复合几何对象，由行、列和单元格组成。

use crate::math::{BoundingBox2, Point2, EPSILON};
use serde::{Deserialize, Serialize};

/// 表格单元格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCell {
    /// 单元格内容
    pub content: String,
    /// 文本对齐方式
    pub alignment: CellAlignment,
    /// 背景颜色（可选，0表示无背景）
    pub background_color: u32,
}

impl Default for TableCell {
    fn default() -> Self {
        Self {
            content: String::new(),
            alignment: CellAlignment::MiddleCenter,
            background_color: 0,
        }
    }
}

impl TableCell {
    /// 创建新的单元格
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            ..Default::default()
        }
    }

    /// 设置对齐方式
    pub fn with_alignment(mut self, alignment: CellAlignment) -> Self {
        self.alignment = alignment;
        self
    }
}

/// 单元格对齐方式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CellAlignment {
    /// 左上
    TopLeft,
    /// 居中靠上
    TopCenter,
    /// 右上
    TopRight,
    /// 左中
    MiddleLeft,
    /// 居中（默认）
    #[default]
    MiddleCenter,
    /// 右中
    MiddleRight,
    /// 左下
    BottomLeft,
    /// 居中靠下
    BottomCenter,
    /// 右下
    BottomRight,
}

/// 表格样式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableStyle {
    /// 行高
    pub row_height: f64,
    /// 默认列宽
    pub column_width: f64,
    /// 文本高度
    pub text_height: f64,
    /// 单元格边距
    pub cell_margin: f64,
    /// 是否显示网格线
    pub show_grid: bool,
    /// 标题行数量（顶部固定行）
    pub header_rows: usize,
    /// 边框宽度
    pub border_width: f64,
}

impl Default for TableStyle {
    fn default() -> Self {
        Self {
            row_height: 5.0,
            column_width: 20.0,
            text_height: 2.5,
            cell_margin: 1.0,
            show_grid: true,
            header_rows: 1,
            border_width: 0.5,
        }
    }
}

/// 表格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    /// 插入点（左上角）
    pub position: Point2,
    /// 行数
    pub rows: usize,
    /// 列数
    pub columns: usize,
    /// 列宽度（每列可不同）
    pub column_widths: Vec<f64>,
    /// 行高度（每行可不同）
    pub row_heights: Vec<f64>,
    /// 单元格数据（row-major顺序）
    pub cells: Vec<TableCell>,
    /// 表格样式
    pub style: TableStyle,
    /// 旋转角度（弧度）
    pub rotation: f64,
}

impl Table {
    /// 创建新的表格
    pub fn new(position: Point2, rows: usize, columns: usize) -> Self {
        let style = TableStyle::default();
        let column_widths = vec![style.column_width; columns];
        let row_heights = vec![style.row_height; rows];
        let cells = vec![TableCell::default(); rows * columns];

        Self {
            position,
            rows,
            columns,
            column_widths,
            row_heights,
            cells,
            style,
            rotation: 0.0,
        }
    }

    /// 创建带标题行的表格
    pub fn with_headers(position: Point2, headers: Vec<String>, data_rows: usize) -> Self {
        let columns = headers.len();
        let rows = data_rows + 1; // 1 for header
        let mut table = Self::new(position, rows, columns);

        // 设置标题
        for (i, header) in headers.into_iter().enumerate() {
            table.set_cell(0, i, TableCell::new(header));
        }

        table
    }

    /// 获取单元格（行，列）
    pub fn get_cell(&self, row: usize, col: usize) -> Option<&TableCell> {
        if row < self.rows && col < self.columns {
            Some(&self.cells[row * self.columns + col])
        } else {
            None
        }
    }

    /// 设置单元格
    pub fn set_cell(&mut self, row: usize, col: usize, cell: TableCell) {
        if row < self.rows && col < self.columns {
            self.cells[row * self.columns + col] = cell;
        }
    }

    /// 设置单元格内容
    pub fn set_cell_content(&mut self, row: usize, col: usize, content: impl Into<String>) {
        if row < self.rows && col < self.columns {
            self.cells[row * self.columns + col].content = content.into();
        }
    }

    /// 获取表格总宽度
    pub fn total_width(&self) -> f64 {
        self.column_widths.iter().sum()
    }

    /// 获取表格总高度
    pub fn total_height(&self) -> f64 {
        self.row_heights.iter().sum()
    }

    /// 设置列宽
    pub fn set_column_width(&mut self, col: usize, width: f64) {
        if col < self.columns {
            self.column_widths[col] = width;
        }
    }

    /// 设置行高
    pub fn set_row_height(&mut self, row: usize, height: f64) {
        if row < self.rows {
            self.row_heights[row] = height;
        }
    }

    /// 获取单元格的左上角位置（相对于表格插入点）
    pub fn cell_position(&self, row: usize, col: usize) -> Point2 {
        let x: f64 = self.column_widths[..col].iter().sum();
        let y: f64 = self.row_heights[..row].iter().sum();
        // 表格从上往下绘制，所以 Y 是负的
        Point2::new(self.position.x + x, self.position.y - y)
    }

    /// 获取单元格的尺寸
    pub fn cell_size(&self, row: usize, col: usize) -> (f64, f64) {
        if row < self.rows && col < self.columns {
            (self.column_widths[col], self.row_heights[row])
        } else {
            (0.0, 0.0)
        }
    }

    /// 添加行
    pub fn add_row(&mut self) {
        self.rows += 1;
        self.row_heights.push(self.style.row_height);
        for _ in 0..self.columns {
            self.cells.push(TableCell::default());
        }
    }

    /// 添加列
    pub fn add_column(&mut self) {
        self.columns += 1;
        self.column_widths.push(self.style.column_width);
        // 需要在每行末尾插入新单元格
        let mut new_cells = Vec::with_capacity(self.rows * self.columns);
        for row in 0..self.rows {
            let start = row * (self.columns - 1);
            let end = start + (self.columns - 1);
            new_cells.extend_from_slice(&self.cells[start..end]);
            new_cells.push(TableCell::default());
        }
        self.cells = new_cells;
    }

    /// 获取包围盒
    pub fn bounding_box(&self) -> BoundingBox2 {
        let width = self.total_width();
        let height = self.total_height();

        if self.rotation.abs() < EPSILON {
            // 无旋转
            BoundingBox2::new(
                Point2::new(self.position.x, self.position.y - height),
                Point2::new(self.position.x + width, self.position.y),
            )
        } else {
            // 有旋转：计算四个角点
            let corners = [
                Point2::new(0.0, 0.0),
                Point2::new(width, 0.0),
                Point2::new(width, -height),
                Point2::new(0.0, -height),
            ];

            let cos_r = self.rotation.cos();
            let sin_r = self.rotation.sin();

            let rotated: Vec<Point2> = corners
                .iter()
                .map(|p| {
                    let rx = p.x * cos_r - p.y * sin_r + self.position.x;
                    let ry = p.x * sin_r + p.y * cos_r + self.position.y;
                    Point2::new(rx, ry)
                })
                .collect();

            BoundingBox2::from_points(rotated)
        }
    }

    /// 检查点是否在表格内
    pub fn contains_point(&self, point: &Point2, tolerance: f64) -> bool {
        let bbox = self.bounding_box();
        let expanded = BoundingBox2::new(
            Point2::new(bbox.min.x - tolerance, bbox.min.y - tolerance),
            Point2::new(bbox.max.x + tolerance, bbox.max.y + tolerance),
        );
        expanded.contains(point)
    }

    /// 根据点击位置获取单元格索引
    pub fn cell_at_point(&self, point: &Point2) -> Option<(usize, usize)> {
        // 简化版本：不考虑旋转
        let rel_x = point.x - self.position.x;
        let rel_y = self.position.y - point.y; // 注意 Y 轴方向

        if rel_x < 0.0 || rel_y < 0.0 {
            return None;
        }

        let mut col = 0;
        let mut x_sum = 0.0;
        for (i, &w) in self.column_widths.iter().enumerate() {
            if rel_x < x_sum + w {
                col = i;
                break;
            }
            x_sum += w;
            if i == self.columns - 1 && rel_x >= x_sum {
                return None; // 超出表格右边界
            }
        }

        let mut row = 0;
        let mut y_sum = 0.0;
        for (i, &h) in self.row_heights.iter().enumerate() {
            if rel_y < y_sum + h {
                row = i;
                break;
            }
            y_sum += h;
            if i == self.rows - 1 && rel_y >= y_sum {
                return None; // 超出表格下边界
            }
        }

        Some((row, col))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_creation() {
        let table = Table::new(Point2::new(0.0, 100.0), 3, 4);
        assert_eq!(table.rows, 3);
        assert_eq!(table.columns, 4);
        assert_eq!(table.cells.len(), 12);
    }

    #[test]
    fn test_table_with_headers() {
        let table = Table::with_headers(
            Point2::new(0.0, 100.0),
            vec!["A".to_string(), "B".to_string(), "C".to_string()],
            2,
        );
        assert_eq!(table.rows, 3);
        assert_eq!(table.columns, 3);
        assert_eq!(table.get_cell(0, 0).unwrap().content, "A");
    }

    #[test]
    fn test_cell_access() {
        let mut table = Table::new(Point2::new(0.0, 100.0), 2, 2);
        table.set_cell_content(0, 0, "Hello");
        table.set_cell_content(1, 1, "World");

        assert_eq!(table.get_cell(0, 0).unwrap().content, "Hello");
        assert_eq!(table.get_cell(1, 1).unwrap().content, "World");
        assert_eq!(table.get_cell(0, 1).unwrap().content, "");
    }

    #[test]
    fn test_table_dimensions() {
        let table = Table::new(Point2::new(0.0, 100.0), 3, 4);
        // 默认列宽 20.0，行高 5.0
        assert_eq!(table.total_width(), 80.0);
        assert_eq!(table.total_height(), 15.0);
    }
}
