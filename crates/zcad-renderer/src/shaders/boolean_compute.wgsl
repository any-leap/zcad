// 布尔运算计算着色器
// 执行几何体的布尔运算（并集、交集、差集、异或）

struct GpuGeometryData {
    geometry_type: u32, // 0: empty, 1: line, 2: circle, 3: point, 4: polyline vertex, 5: arc
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    radius: f32,
    bulge: f32,
    param1: f32,
    param2: f32,
};

struct BooleanParams {
    operation: u32,      // 0: union, 1: intersection, 2: difference, 3: xor
    tolerance: f32,
    input1_count: u32,
    input2_count: u32,
};

// 绑定组
@group(0) @binding(0)
var<storage, read> input_geoms1: array<GpuGeometryData>;

@group(0) @binding(1)
var<storage, read> input_geoms2: array<GpuGeometryData>;

@group(0) @binding(2)
var<storage, read_write> output_geoms: array<GpuGeometryData>;

@group(0) @binding(3)
var<uniform> params: BooleanParams;

// 计算两点距离
fn distance(p1: vec2<f32>, p2: vec2<f32>) -> f32 {
    return length(p1 - p2);
}

// 检查点是否在圆内
fn point_in_circle(point: vec2<f32>, center: vec2<f32>, radius: f32) -> bool {
    return distance(point, center) <= radius + params.tolerance;
}

// 检查点是否在线段上
fn point_on_line(point: vec2<f32>, start: vec2<f32>, end: vec2<f32>) -> bool {
    let v = end - start;
    let w = point - start;

    let c1 = dot(w, v);
    if c1 <= 0.0 {
        return distance(point, start) <= params.tolerance;
    }

    let c2 = dot(v, v);
    if c2 <= c1 {
        return distance(point, end) <= params.tolerance;
    }

    let b = c1 / c2;
    let pb = start + b * v;
    return distance(point, pb) <= params.tolerance;
}

// 简化的几何相交测试
fn geometries_intersect(g1: GpuGeometryData, g2: GpuGeometryData) -> bool {
    let p1 = vec2<f32>(g1.x1, g1.y1);
    let p2 = vec2<f32>(g1.x2, g1.y2);
    let center1 = vec2<f32>(g1.x1, g1.y1);
    let radius1 = g1.radius;

    let q1 = vec2<f32>(g2.x1, g2.y1);
    let q2 = vec2<f32>(g2.x2, g2.y2);
    let center2 = vec2<f32>(g2.x1, g2.y1);
    let radius2 = g2.radius;

    // 线段-线段相交
    if g1.geometry_type == 1u && g2.geometry_type == 1u {
        // 简化的线段相交测试（实际应该用更精确的算法）
        let dist1 = point_on_line(p1, q1, q2) || point_on_line(p2, q1, q2);
        let dist2 = point_on_line(q1, p1, p2) || point_on_line(q2, p1, p2);
        return dist1 || dist2;
    }

    // 圆-圆相交
    if g1.geometry_type == 2u && g2.geometry_type == 2u {
        let dist = distance(center1, center2);
        return dist <= (radius1 + radius2 + params.tolerance) && dist >= abs(radius1 - radius2) - params.tolerance;
    }

    // 线段-圆相交
    if (g1.geometry_type == 1u && g2.geometry_type == 2u) || (g1.geometry_type == 2u && g2.geometry_type == 1u) {
        let line_start = select(p1, q1, g1.geometry_type == 2u);
        let line_end = select(p2, q2, g1.geometry_type == 2u);
        let circle_center = select(center2, center1, g1.geometry_type == 2u);
        let circle_radius = select(radius2, radius1, g1.geometry_type == 2u);

        // 计算点到线段的距离
        let v = line_end - line_start;
        let w = circle_center - line_start;
        let c1 = dot(w, v);
        var dist = 0.0;

        if c1 <= 0.0 {
            dist = distance(circle_center, line_start);
        } else {
            let c2 = dot(v, v);
            if c2 <= c1 {
                dist = distance(circle_center, line_end);
            } else {
                let b = c1 / c2;
                let pb = line_start + b * v;
                dist = distance(circle_center, pb);
            }
        }

        return dist <= circle_radius + params.tolerance;
    }

    return false;
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;

    // 每个工作组处理一个几何体对
    if idx >= params.input1_count * params.input2_count {
        return;
    }

    let i = idx / params.input2_count;
    let j = idx % params.input2_count;

    if i >= params.input1_count || j >= params.input2_count {
        return;
    }

    let geom1 = input_geoms1[i];
    let geom2 = input_geoms2[j];

    // 根据操作类型决定输出
    var should_output = false;
    var output_geom = GpuGeometryData(0u, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);

    switch params.operation {
        case 0u: { // Union - 并集
            // 如果几何体不相交，输出两者
            should_output = true;
            if i == 0u {
                output_geom = geom1;
            } else {
                output_geom = geom2;
            }
        }
        case 1u: { // Intersection - 交集
            // 只在相交时输出
            if geometries_intersect(geom1, geom2) {
                should_output = true;
                output_geom = geom1; // 简化为输出第一个几何体
            }
        }
        case 2u: { // Difference - 差集
            // 只输出第一个几何体（减去相交部分）
            if i < params.input1_count {
                should_output = true;
                output_geom = geom1;
            }
        }
        case 3u: { // Xor - 异或
            // 输出不相交的部分
            if !geometries_intersect(geom1, geom2) {
                should_output = true;
                output_geom = geom1;
            }
        }
        default: {}
    }

    if should_output {
        // 找到输出位置（简化：直接使用索引）
        let output_idx = idx;
        if output_idx < arrayLength(&output_geoms) {
            output_geoms[output_idx] = output_geom;
        }
    }
}
