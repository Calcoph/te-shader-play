struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>
};

struct VertexInput {
    @location(0) pos: vec3<f32>
}

struct Camera {
    @location(0) pos: vec3<f32>,
    @location(1) projection: mat4x4<f32>,
    @location(2) view: mat4x4<f32>
}

@group(0) @binding(0)
var<uniform> millis: u32;
@group(1) @binding(0)
var<uniform> camera: Camera;

@vertex
fn vs_main(inp: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    var pos = inp.pos;

    out.uv = pos.xz;
    out.clip_position = camera.projection * camera.view * vec4<f32>(pos.x, pos.z, pos.y, 1.0);
    return out;
}


// Fragment shader credit: kishimisu on youtube
// copied from:
// https://www.youtube.com/watch?v=f4s1h2YETNY
@fragment
fn fs_main(coord_in: VertexOutput) -> @location(0) vec4<f32> {
    let i_time = f32(millis) / 1000.0;
    var uv = coord_in.uv;
    let uv0 = uv;
    var final_color = vec3(0.0);

    for (var i: f32 = 0.0; i<4.0; i += 1.0){
        uv *= 1.5;
        uv = fract(uv);
        uv -= 0.5;

        var d0 = length(uv0);
        var d = length(uv) * exp(-d0);

        var col = palette(d0 + i * 0.4 + i_time);

        d = sin(d * 8.0 + i_time) / 8.0;
        d = abs(d);

        d = 0.01 / d;

        d = pow(d, 1.2);

        col *= d;

        final_color += col * d;
    }

    //uv *= pow(((uv + 0.055) / 1.055 ),vec2(2.4,2.4));
    return vec4<f32>(final_color, 1.0);
}

fn palette(t: f32) -> vec3<f32> {
    let a = vec3<f32>(0.5, 0.5, 0.5);
    let b = vec3<f32>(0.5, 0.5, 0.5);
    let c = vec3<f32>(1.0, 1.0, 1.0);
    let d = vec3<f32>(0.263, 0.416, 0.557);

    return a + b * cos(6.28318*(c*t+d));
}
