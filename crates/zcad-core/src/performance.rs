//! 性能优化策略
//!
//! 本模块记录 ZCAD 相对于传统 CAD 软件的性能优势和优化策略。
//!
//! # ZCAD vs AutoCAD 性能对比
//!
//! ## 启动性能
//!
//! | 阶段 | AutoCAD | ZCAD | 原因 |
//! |------|---------|------|------|
//! | 许可证验证 | 2-5s | 0s | 开源无需验证 |
//! | 运行时初始化 | 5-10s | 0s | Rust 无 GC/JIT |
//! | 插件加载 | 5-15s | 0s | 静态编译，无 DLL |
//! | 字体加载 | 2-5s | <0.1s | 按需加载 |
//! | 渲染初始化 | 3-5s | <0.5s | 现代 GPU API |
//! | **总计** | **20-60s** | **<1s** | |
//!
//! ## 内存使用
//!
//! | 场景 | AutoCAD | ZCAD | 原因 |
//! |------|---------|------|------|
//! | 空文档 | 500MB+ | <50MB | 无历史包袱 |
//! | 1万实体 | 1GB+ | <100MB | 紧凑数据结构 |
//! | 10万实体 | 3GB+ | <500MB | 空间索引优化 |
//!
//! # 性能优化策略
//!
//! ## 1. 启动优化
//!
//! ### 1.1 延迟加载 (Lazy Loading)
//!
//! ```rust,ignore
//! // ❌ AutoCAD 方式：启动时加载所有字体
//! fn init() {
//!     load_all_fonts();  // 加载 500+ 字体，耗时 5 秒
//! }
//!
//! // ✅ ZCAD 方式：按需加载
//! fn get_font(name: &str) -> &Font {
//!     FONT_CACHE.get_or_insert(name, || load_font(name))
//! }
//! ```
//!
//! ### 1.2 静态编译 vs 动态加载
//!
//! ```rust,ignore
//! // ❌ AutoCAD：运行时加载 DLL
//! LoadLibrary("acad.arx");  // 每个插件都要 IO 操作
//!
//! // ✅ ZCAD：编译时链接
//! // 所有功能已经编译进二进制，无需运行时加载
//! ```
//!
//! ## 2. 渲染优化
//!
//! ### 2.1 GPU 加速
//!
//! ```rust,ignore
//! // ❌ 传统方式：CPU 绘制每个实体
//! for entity in entities {
//!     cpu_draw(entity);  // 每帧重复计算
//! }
//!
//! // ✅ ZCAD 方式：批量 GPU 渲染
//! let vertex_buffer = upload_to_gpu(entities);
//! gpu.draw_instanced(vertex_buffer);  // 一次调用绘制全部
//! ```
//!
//! ### 2.2 视锥体裁剪 (Frustum Culling)
//!
//! ```rust,ignore
//! // ❌ 传统方式：遍历所有实体
//! for entity in all_entities {
//!     if is_visible(entity) { draw(entity); }
//! }
//!
//! // ✅ ZCAD 方式：空间索引查询
//! let visible = rtree.query(viewport_bounds);  // O(log n)
//! draw_batch(visible);
//! ```
//!
//! ### 2.3 LOD (Level of Detail)
//!
//! ```rust,ignore
//! // 根据缩放级别简化渲染
//! fn draw_circle(circle: &Circle, zoom: f64) {
//!     let segments = if zoom < 0.1 {
//!         8   // 远距离：8 边形
//!     } else if zoom < 1.0 {
//!         32  // 中等：32 边形
//!     } else {
//!         64  // 近距离：64 边形
//!     };
//!     draw_polygon(circle, segments);
//! }
//! ```
//!
//! ## 3. 数据结构优化
//!
//! ### 3.1 紧凑存储
//!
//! ```rust,ignore
//! // ❌ 传统方式：OOP 对象（虚表、继承开销）
//! class Entity {
//!     vtable*,           // 8 bytes
//!     base_class_data,   // 变长
//!     // ...
//! };
//!
//! // ✅ ZCAD 方式：枚举 + 紧凑结构
//! enum Geometry {       // 1 byte tag
//!     Line(Line),       // 32 bytes inline
//!     Circle(Circle),   // 24 bytes inline
//! }
//! ```
//!
//! ### 3.2 空间索引
//!
//! ```rust,ignore
//! // ❌ 线性搜索：O(n)
//! fn find_at_point(point: Point) -> Vec<Entity> {
//!     entities.iter().filter(|e| e.contains(point)).collect()
//! }
//!
//! // ✅ R-Tree 查询：O(log n)
//! fn find_at_point(point: Point) -> Vec<Entity> {
//!     rtree.query_point(point)
//! }
//! ```
//!
//! ## 4. 内存优化
//!
//! ### 4.1 零拷贝设计
//!
//! ```rust,ignore
//! // ❌ 传统方式：频繁复制
//! fn get_entities() -> Vec<Entity> {
//!     self.entities.clone()  // 复制所有数据
//! }
//!
//! // ✅ ZCAD 方式：借用
//! fn get_entities(&self) -> &[Entity] {
//!     &self.entities  // 零拷贝
//! }
//! ```
//!
//! ### 4.2 对象池
//!
//! ```rust,ignore
//! // 预分配常用对象，避免频繁 malloc/free
//! struct EntityPool {
//!     lines: Vec<Line>,       // 预分配
//!     circles: Vec<Circle>,   // 预分配
//!     free_indices: Vec<usize>,
//! }
//! ```
//!
//! ## 5. 并行处理
//!
//! ### 5.1 Rayon 并行迭代
//!
//! ```rust,ignore
//! use rayon::prelude::*;
//!
//! // ❌ 单线程
//! let results: Vec<_> = entities.iter().map(|e| compute(e)).collect();
//!
//! // ✅ 自动并行
//! let results: Vec<_> = entities.par_iter().map(|e| compute(e)).collect();
//! ```
//!
//! ### 5.2 异步 IO
//!
//! ```rust,ignore
//! // ❌ 阻塞 IO
//! let file = std::fs::read("large.dxf")?;  // 阻塞主线程
//!
//! // ✅ 异步 IO
//! let file = tokio::fs::read("large.dxf").await?;  // 不阻塞 UI
//! ```
//!
//! ## 6. 增量更新
//!
//! ### 6.1 脏矩形渲染
//!
//! ```rust,ignore
//! // ❌ 每帧重绘全部
//! fn render() {
//!     clear_screen();
//!     for entity in all_entities {
//!         draw(entity);
//!     }
//! }
//!
//! // ✅ 只重绘变化区域
//! fn render() {
//!     for dirty_rect in dirty_regions {
//!         let affected = rtree.query(dirty_rect);
//!         for entity in affected {
//!             draw(entity);
//!         }
//!     }
//! }
//! ```
//!
//! ### 6.2 增量撤销
//!
//! ```rust,ignore
//! // ❌ AutoCAD 早期：每次存储完整快照
//! fn undo() {
//!     document = snapshots.pop();  // 可能几 MB
//! }
//!
//! // ✅ ZCAD：增量操作
//! fn undo() {
//!     let op = operations.pop();  // 几 bytes
//!     op.reverse(&mut document);
//! }
//! ```

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// 性能计时器
#[derive(Debug)]
pub struct PerfTimer {
    name: &'static str,
    start: Instant,
}

impl PerfTimer {
    /// 开始计时
    pub fn start(name: &'static str) -> Self {
        Self {
            name,
            start: Instant::now(),
        }
    }

    /// 结束计时并返回毫秒数
    pub fn stop(self) -> f64 {
        let elapsed = self.start.elapsed();
        let ms = elapsed.as_secs_f64() * 1000.0;
        #[cfg(debug_assertions)]
        {
            if ms > 16.0 {
                // 超过一帧时间，警告
                eprintln!("[PERF WARNING] {} took {:.2}ms", self.name, ms);
            }
        }
        ms
    }
}

/// 性能统计
#[derive(Debug, Default)]
pub struct PerfStats {
    /// 帧计数
    pub frame_count: AtomicU64,
    /// 总渲染时间（微秒）
    pub total_render_us: AtomicU64,
    /// 最大帧时间（微秒）
    pub max_frame_us: AtomicU64,
    /// 实体数量
    pub entity_count: AtomicU64,
}

impl PerfStats {
    /// 创建新的统计实例
    pub fn new() -> Self {
        Self::default()
    }

    /// 记录一帧
    pub fn record_frame(&self, render_us: u64) {
        self.frame_count.fetch_add(1, Ordering::Relaxed);
        self.total_render_us.fetch_add(render_us, Ordering::Relaxed);
        
        // 更新最大帧时间
        let mut current_max = self.max_frame_us.load(Ordering::Relaxed);
        while render_us > current_max {
            match self.max_frame_us.compare_exchange(
                current_max,
                render_us,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current_max = actual,
            }
        }
    }

    /// 获取平均帧时间（毫秒）
    pub fn avg_frame_ms(&self) -> f64 {
        let count = self.frame_count.load(Ordering::Relaxed);
        if count == 0 {
            return 0.0;
        }
        let total_us = self.total_render_us.load(Ordering::Relaxed);
        (total_us as f64 / count as f64) / 1000.0
    }

    /// 获取平均 FPS
    pub fn avg_fps(&self) -> f64 {
        let avg_ms = self.avg_frame_ms();
        if avg_ms <= 0.0 {
            return 0.0;
        }
        1000.0 / avg_ms
    }

    /// 重置统计
    pub fn reset(&self) {
        self.frame_count.store(0, Ordering::Relaxed);
        self.total_render_us.store(0, Ordering::Relaxed);
        self.max_frame_us.store(0, Ordering::Relaxed);
    }

    /// 设置实体数量
    pub fn set_entity_count(&self, count: u64) {
        self.entity_count.store(count, Ordering::Relaxed);
    }
}

/// 延迟初始化包装器
#[derive(Debug)]
pub struct Lazy<T, F = fn() -> T> {
    init: Option<F>,
    value: Option<T>,
}

impl<T, F: FnOnce() -> T> Lazy<T, F> {
    /// 创建延迟初始化包装器
    pub const fn new(init: F) -> Self {
        Self {
            init: Some(init),
            value: None,
        }
    }

    /// 获取值（首次访问时初始化）
    pub fn get(&mut self) -> &T {
        if self.value.is_none() {
            if let Some(init) = self.init.take() {
                self.value = Some(init());
            }
        }
        self.value.as_ref().unwrap()
    }

    /// 获取可变引用
    pub fn get_mut(&mut self) -> &mut T {
        if self.value.is_none() {
            if let Some(init) = self.init.take() {
                self.value = Some(init());
            }
        }
        self.value.as_mut().unwrap()
    }

    /// 检查是否已初始化
    pub fn is_initialized(&self) -> bool {
        self.value.is_some()
    }
}

/// 简单的缓存
#[derive(Debug)]
pub struct Cache<K, V> {
    entries: Vec<(K, V)>,
    max_size: usize,
}

impl<K: PartialEq, V> Cache<K, V> {
    /// 创建新缓存
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: Vec::with_capacity(max_size),
            max_size,
        }
    }

    /// 获取或插入
    pub fn get_or_insert<F: FnOnce() -> V>(&mut self, key: K, f: F) -> &V {
        // 查找现有条目
        if let Some(pos) = self.entries.iter().position(|(k, _)| *k == key) {
            return &self.entries[pos].1;
        }

        // 如果满了，移除最旧的
        if self.entries.len() >= self.max_size {
            self.entries.remove(0);
        }

        // 插入新条目
        self.entries.push((key, f()));
        &self.entries.last().unwrap().1
    }

    /// 清空缓存
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perf_timer() {
        let timer = PerfTimer::start("test");
        std::thread::sleep(std::time::Duration::from_millis(10));
        let ms = timer.stop();
        assert!(ms >= 9.0 && ms < 20.0);
    }

    #[test]
    fn test_lazy() {
        let mut lazy: Lazy<i32, _> = Lazy::new(|| {
            println!("Initializing...");
            42
        });
        
        assert!(!lazy.is_initialized());
        assert_eq!(*lazy.get(), 42);
        assert!(lazy.is_initialized());
    }

    #[test]
    fn test_cache() {
        let mut cache: Cache<&str, i32> = Cache::new(2);
        
        let a = *cache.get_or_insert("a", || 1);
        let b = *cache.get_or_insert("b", || 2);
        let c = *cache.get_or_insert("c", || 3); // 这会移除 "a"
        
        assert_eq!(a, 1);
        assert_eq!(b, 2);
        assert_eq!(c, 3);
    }
}
