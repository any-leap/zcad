// 几何变换计算着色器
// 执行几何体的平移、旋转、缩放变换

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

struct TransformMatrix {
    m11: f32, m12: f32, m13: f32,
    m21: f32, m22: f32, m23: f32,
    m31: f32, m32: f32, m33: f32,
};

// 绑定组
@group(0) @binding(0)
var<storage, read> input_geoms: array<GpuGeometryData>;

@group(0) @binding(1)
var<storage, read_write> output_geoms: array<GpuGeometryData>;

@group(0) @binding(2)
var<uniform> transform: TransformMatrix;

// 应用2D变换到点
fn transform_point(x: f32, y: f32) -> vec2<f32> {
    let new_x = transform.m11 * x + transform.m12 * y + transform.m13;
    let new_y = transform.m21 * x + transform.m22 * y + transform.m23;
    return vec2<f32>(new_x, new_y);
}

// 变换线段
fn transform_line(line: GpuGeometryData) -> GpuGeometryData {
    let start = transform_point(line.x1, line.y1);
    let end = transform_point(line.x2, line.y2);

    return GpuGeometryData(
        1u, // line
        start.x, start.y,
        end.x, end.y,
        line.radius,
        line.bulge,
        line.param1, line.param2
    );
}

// 变换圆
fn transform_circle(circle: GpuGeometryData) -> GpuGeometryData {
    let center = transform_point(circle.x1, circle.y1);

    // 计算缩放因子（假设均匀缩放）
    let scale_x = length(vec2<f32>(transform.m11, transform.m21));
    let scale_y = length(vec2<f32>(transform.m12, transform.m22));
    let scale = (scale_x + scale_y) * 0.5;

    return GpuGeometryData(
        2u, // circle
        center.x, center.y,
        0.0, 0.0,
        circle.radius * scale,
        circle.bulge,
        circle.param1, circle.param2
    );
}

// 变换点
fn transform_point_geom(point: GpuGeometryData) -> GpuGeometryData {
    let new_pos = transform_point(point.x1, point.y1);

    return GpuGeometryData(
        3u, // point
        new_pos.x, new_pos.y,
        0.0, 0.0,
        point.radius,
        point.bulge,
        point.param1, point.param2
    );
}

// 变换圆弧
fn transform_arc(arc: GpuGeometryData) -> GpuGeometryData {
    let center = transform_point(arc.x1, arc.y1);

    // 计算缩放因子
    let scale_x = length(vec2<f32>(transform.m11, transform.m21));
    let scale_y = length(vec2<f32>(transform.m12, transform.m22));
    let scale = (scale_x + scale_y) * 0.5;

    // 计算旋转角度（从变换矩阵提取）
    let rotation = atan2(transform.m12, transform.m11);

    return GpuGeometryData(
        5u, // arc
        center.x, center.y,
        0.0, 0.0,
        arc.radius * scale,
        arc.bulge,
        arc.param1 + rotation, // 起始角度
        arc.param2 + rotation  // 结束角度
    );
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;

    if idx >= arrayLength(&input_geoms) {
        return;
    }

    let geom = input_geoms[idx];
    var result_geom = GpuGeometryData(0u, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);

    switch geom.geometry_type {
        case 1u: { // Line
            result_geom = transform_line(geom);
        }
        case 2u: { // Circle
            result_geom = transform_circle(geom);
        }
        case 3u: { // Point
            result_geom = transform_point_geom(geom);
        }
        case 4u: { // Polyline vertex
            let new_pos = transform_point(geom.x1, geom.y1);
            result_geom = GpuGeometryData(
                4u, // polyline vertex
                new_pos.x, new_pos.y,
                0.0, 0.0,
                geom.radius,
                geom.bulge,
                geom.param1, geom.param2
            );
        }
        case 5u: { // Arc
            result_geom = transform_arc(geom);
        }
        default: {
            // 未知类型，复制原样
            result_geom = geom;
        }
    }

    output_geoms[idx] = result_geom;
}
