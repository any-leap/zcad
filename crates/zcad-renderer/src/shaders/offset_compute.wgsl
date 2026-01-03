// 偏移运算计算着色器
// 执行几何体的偏移/膨胀/腐蚀操作

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

struct OffsetParams {
    distance: f32,
    tolerance: f32,
    input_count: u32,
};

// 绑定组
@group(0) @binding(0)
var<storage, read> input_geoms: array<GpuGeometryData>;

@group(0) @binding(1)
var<storage, read_write> output_geoms: array<GpuGeometryData>;

@group(0) @binding(2)
var<uniform> params: OffsetParams;

// 计算两点距离
fn distance(p1: vec2<f32>, p2: vec2<f32>) -> f32 {
    return length(p1 - p2);
}

// 计算2D向量的垂直向量
fn perpendicular(v: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(-v.y, v.x);
}

// 归一化向量
fn normalize_safe(v: vec2<f32>) -> vec2<f32> {
    let len = length(v);
    if len < 0.0001 {
        return vec2<f32>(0.0, 0.0);
    }
    return v / len;
}

// 线段偏移
fn offset_line(line: GpuGeometryData, distance: f32) -> array<GpuGeometryData, 2> {
    let start = vec2<f32>(line.x1, line.y1);
    let end = vec2<f32>(line.x2, line.y2);
    let dir = normalize_safe(end - start);
    let perp = perpendicular(dir) * distance;

    var result: array<GpuGeometryData, 2>;
    result[0] = GpuGeometryData(
        1u, // line
        start.x + perp.x, start.y + perp.y,
        end.x + perp.x, end.y + perp.y,
        0.0, 0.0, 0.0, 0.0
    );
    result[1] = result[0]; // 简化：只返回一条线段

    return result;
}

// 圆偏移
fn offset_circle(circle: GpuGeometryData, distance: f32) -> GpuGeometryData {
    return GpuGeometryData(
        2u, // circle
        circle.x1, circle.y1, // center
        0.0, 0.0,
        circle.radius + distance,
        0.0, 0.0, 0.0
    );
}

// 圆弧偏移
fn offset_arc(arc: GpuGeometryData, distance: f32) -> GpuGeometryData {
    return GpuGeometryData(
        5u, // arc
        arc.x1, arc.y1, // center
        0.0, 0.0,
        arc.radius + distance,
        arc.bulge,
        arc.param1, arc.param2 // start/end angles
    );
}

// 多段线顶点偏移
fn offset_polyline_vertex(vertex: GpuGeometryData, prev_vertex: GpuGeometryData, next_vertex: GpuGeometryData, distance: f32) -> GpuGeometryData {
    let current = vec2<f32>(vertex.x1, vertex.y1);
    let prev = vec2<f32>(prev_vertex.x1, prev_vertex.y1);
    let next = vec2<f32>(next_vertex.x1, next_vertex.y1);

    // 计算前后线段的方向
    let dir_prev = normalize_safe(current - prev);
    let dir_next = normalize_safe(next - current);

    // 计算平均方向和垂直方向
    let avg_dir = normalize_safe(dir_prev + dir_next);
    let perp = perpendicular(avg_dir) * distance;

    return GpuGeometryData(
        4u, // polyline vertex
        current.x + perp.x, current.y + perp.y,
        0.0, 0.0,
        0.0,
        vertex.bulge,
        vertex.param1, vertex.param2
    );
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;

    if idx >= params.input_count {
        return;
    }

    let geom = input_geoms[idx];
    let output_idx = idx;

    if output_idx >= arrayLength(&output_geoms) {
        return;
    }

    var result_geom = GpuGeometryData(0u, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);

    switch geom.geometry_type {
        case 1u: { // Line
            let offset_lines = offset_line(geom, params.distance);
            result_geom = offset_lines[0];
        }
        case 2u: { // Circle
            result_geom = offset_circle(geom, params.distance);
        }
        case 3u: { // Point - 点偏移为小圆
            result_geom = GpuGeometryData(
                2u, // circle
                geom.x1, geom.y1,
                0.0, 0.0,
                abs(params.distance), // 使用距离作为半径
                0.0, 0.0, 0.0
            );
        }
        case 4u: { // Polyline vertex
            // 需要前后的顶点信息，这里简化处理
            if idx > 0u && idx < params.input_count - 1u {
                let prev_geom = input_geoms[idx - 1u];
                let next_geom = input_geoms[idx + 1u];
                result_geom = offset_polyline_vertex(geom, prev_geom, next_geom, params.distance);
            } else {
                // 端点：简单偏移
                result_geom = GpuGeometryData(
                    3u, // point
                    geom.x1 + params.distance, geom.y1, // 简化的偏移
                    0.0, 0.0,
                    0.0, 0.0, 0.0, 0.0
                );
            }
        }
        case 5u: { // Arc
            result_geom = offset_arc(geom, params.distance);
        }
        default: {
            // 未知类型，复制原样
            result_geom = geom;
        }
    }

    output_geoms[output_idx] = result_geom;
}
