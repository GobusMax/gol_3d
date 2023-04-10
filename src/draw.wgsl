struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) shade: f32,
    @location(2) state: u32,
};

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>
}

struct CameraUniform {
    view_proj: mat4x4<f32>
}
struct InstanceInput {
    @location(5) pos: vec3<f32>,
    @location(6) state: u32
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;



@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput
) -> VertexOutput {
    let pos = camera.view_proj * ((vec4<f32>(instance.pos, 1.0) + vec4<f32>(model.position, 1.0)));
    let color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    let shade = (1. + dot(model.normal, normalize(vec3<f32>(1., 2., -1.)))) * 0.5;
    return VertexOutput(pos, color, shade, instance.state);
}



    @fragment
fn fs_main(in: VertexOutput) -> @ location(0) vec4<f32> {
    if in.state == 0u {
        discard;
    } else {
        return in.color * in.shade;
    }
}
