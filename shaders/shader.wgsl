struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>
};

@vertex
fn vs_main(@builtin(vertex_index) v_index: u32, @builtin(instance_index) i_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let impar = i32(v_index) & 1;
    let f_inst = f32(i_index);
    let x = (-1.0 + 2.0*step(0.5, f_inst)) * (f_inst - f32(impar));
    let y = (-1.0 + 2.0*step(0.5, f_inst)) * (f_inst - step(2.0, f32(v_index)));

    out.clip_position = vec4<f32>(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
    out.uv = out.clip_position.xy;
    return out;
}

@group(0) @binding(0)
var<uniform> millis: u32;


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
