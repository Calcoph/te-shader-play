// Source: http://asliceofrendering.com/scene%20helper/2020/01/05/InfiniteGrid/

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) nearPoint: vec3<f32>,
    @location(2) farPoint: vec3<f32>,
};

struct VertexInput {
    @location(0) pos: vec3<f32>
}

struct Camera {
    @location(0) pos: vec3<f32>,
    @location(1) projection: mat4x4<f32>,
    @location(5) view: mat4x4<f32>,
    @location(9) inverse_view: mat4x4<f32>,
    @location(13) inverse_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> millis: u32;
@group(1) @binding(0)
var<uniform> camera: Camera;

fn inverse(matrix: mat4x4<f32>) -> mat4x4<f32> {
    let cof00 = matrix[0].x * determinant(mat3x3(
        matrix[1].yzw,
        matrix[2].yzw,
        matrix[3].yzw,
    ));
    let cof01 = matrix[0].y * determinant(mat3x3(
        matrix[1].xzw,
        matrix[2].xzw,
        matrix[3].xzw,
    ));
    let cof02 = matrix[0].z * determinant(mat3x3(
        matrix[1].xyw,
        matrix[2].xyw,
        matrix[3].xyw,
    ));
    let cof03 = matrix[0].w * determinant(mat3x3(
        matrix[1].xyz,
        matrix[2].xyz,
        matrix[3].xyz,
    ));

    let cof10 = matrix[1].x * determinant(mat3x3(
        matrix[0].yzw,
        matrix[2].yzw,
        matrix[3].yzw,
    ));
    let cof11 = matrix[1].y * determinant(mat3x3(
        matrix[0].xzw,
        matrix[2].xzw,
        matrix[3].xzw,
    ));
    let cof12 = matrix[1].z * determinant(mat3x3(
        matrix[0].xyw,
        matrix[2].xyw,
        matrix[3].xyw,
    ));
    let cof13 = matrix[1].w * determinant(mat3x3(
        matrix[0].xyz,
        matrix[2].xyz,
        matrix[3].xyz,
    ));

    let cof20 = matrix[2].x * determinant(mat3x3(
        matrix[0].yzw,
        matrix[1].yzw,
        matrix[3].yzw,
    ));
    let cof21 = matrix[2].y * determinant(mat3x3(
        matrix[0].xzw,
        matrix[1].xzw,
        matrix[3].xzw,
    ));
    let cof22 = matrix[2].z * determinant(mat3x3(
        matrix[0].xyw,
        matrix[1].xyw,
        matrix[3].xyw,
    ));
    let cof23 = matrix[2].w * determinant(mat3x3(
        matrix[0].xyz,
        matrix[1].xyz,
        matrix[3].xyz,
    ));

    let cof30 = matrix[3].x * determinant(mat3x3(
        matrix[0].yzw,
        matrix[1].yzw,
        matrix[2].yzw,
    ));
    let cof31 = matrix[3].y * determinant(mat3x3(
        matrix[0].xzw,
        matrix[1].xzw,
        matrix[2].xzw,
    ));
    let cof32 = matrix[3].z * determinant(mat3x3(
        matrix[0].xyw,
        matrix[1].xyw,
        matrix[2].xyw,
    ));
    let cof33 = matrix[3].w * determinant(mat3x3(
        matrix[0].xyz,
        matrix[1].xyz,
        matrix[2].xyz,
    ));

    let adjoing = mat4x4(
        vec4(cof00, -cof01, cof02, -cof03),
        vec4(-cof10, cof11, -cof12, cof13),
        vec4(cof20, -cof21, cof22, -cof23),
        vec4(-cof30, cof31, -cof32, cof33),
    );

    return adjoing * (1/determinant(matrix));
}

fn unprojectPoint(x: f32, y: f32, z: f32) -> vec3<f32> {
    let unprojectedPoint = camera.inverse_view * camera.inverse_proj * vec4(x,y,z,1.0);

    return unprojectedPoint.xyz / unprojectedPoint.w;
}

@vertex
fn vs_main(inp: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    var pos = inp.pos;
    out.nearPoint = unprojectPoint(pos.x, pos.y, 0.0);
    out.farPoint = unprojectPoint(pos.x, pos.y, 1.0);
    //out.clip_position = camera.projection * camera.view * vec4<f32>(pos.x, pos.y, pos.z, 1.0);
    out.clip_position = vec4<f32>(pos.x, pos.y, pos.z, 1.0);

    out.uv = pos.xy;//unprojectPoint(out.clip_position.x, out.clip_position.y, 75.0).xy;
    return out;
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @builtin(frag_depth) depth: f32,
}

@fragment
fn fs_main(coord_in: VertexOutput) -> FragmentOutput {
    let t = -coord_in.nearPoint.y / (coord_in.farPoint.y - coord_in.nearPoint.y);
    let fragPos3D = coord_in.nearPoint + t * (coord_in.farPoint - coord_in.nearPoint);
    let linear_depth = computeLinearDepth(fragPos3D);
    let fading = min(1.0, max(0.0, 1.0 - linear_depth));
    var final_color = grid(fragPos3D, 1.0);
    final_color.w *= pow(fading * 30.0, 2.0);
    if t <= 0.0 {
        final_color.w = 0.0;
    }

    var out: FragmentOutput;
    out.color = final_color;
    out.depth = computeDepth(fragPos3D);

    return out;
}

fn grid(fragPos3D: vec3<f32>, scale: f32) -> vec4<f32> {
    let coord = fragPos3D.xz * scale;
    let derivative = fwidth(coord);
    let grid = abs(fract(coord - 0.5) - 0.5) / derivative;
    let line = min(grid.x, grid.y);
    let mininumz = min(derivative.y, 1.0);
    let mininumx = min(derivative.x, 1.0);

    var final_color = vec4(0.2, 0.2, 0.2, 1.0 - min(line, 1.0));

    if fragPos3D.x > -0.50 * mininumx && fragPos3D.x < 0.50 * mininumx {
        final_color.x = 1.0;
    }

    if fragPos3D.z > -0.50 * mininumz && fragPos3D.z < 0.50 * mininumz {
        final_color.z = 1.0;
    }

    return final_color;
}

fn computeDepth(pos: vec3<f32>) -> f32 {
    let clip_space_pos = camera.projection * camera.view * vec4(pos, 1.0);

    return (clip_space_pos.z / clip_space_pos.w);
}

fn computeLinearDepth(pos: vec3<f32>) -> f32 {
    let znear = 0.1;
    let zfar = 100.0;
    let clip_space_pos = camera.projection * camera.view * vec4(pos, 1.0);
    let clip_space_depth = (clip_space_pos.z / clip_space_pos.w) * 2.0 - 1.0;
    let linear_depth = (2.0 * znear * zfar) / (zfar + znear - clip_space_depth * (zfar - znear));

    return clip_space_depth;
    //return linear_depth / zfar;
}

fn palette(t: f32) -> vec3<f32> {
    let a = vec3<f32>(0.5, 0.5, 0.5);
    let b = vec3<f32>(0.5, 0.5, 0.5);
    let c = vec3<f32>(1.0, 1.0, 1.0);
    let d = vec3<f32>(0.263, 0.416, 0.557);

    return a + b * cos(6.28318*(c*t+d));
}
