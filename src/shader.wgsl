// const LIGHT_DIR = vec3<f32>(0.,1.,1.);
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) shade: f32
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
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
    @location(9) color: vec4<f32>
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(instance.model_matrix_0, instance.model_matrix_1, instance.model_matrix_2, instance.model_matrix_3);
    let shade = (1. + dot(model.normal, normalize(vec3<f32>(1., 2., -1.)))) * 0.5;
    return VertexOutput(camera.view_proj * model_matrix * vec4<f32>(model.position, 1.0), model.tex_coords, instance.color, shade);
}


// @group(0) @binding(0)
// var t_diffuse: texture_2d<f32>;

// @group(0) @binding(1)
// var s_diffuse: sampler;


@fragment 
fn fs_main(in: VertexOutput) -> @ location(0) vec4<f32> {
    return in.color * in.shade;
    // return textureSample(t_diffuse, s_diffuse, in.tex_coords);
    // return vec4<f32>(textureSample(t_diffuse, s_diffuse, in.tex_coords).xyz * (1. + dot(normalize(in.normal), normalize(vec3<f32>(1., 0.5, -1.5)))) * 0.5, 1.);
    // return vec4<f32>((1. + dot(normalize(in.normal), normalize(vec3<f32>(1., 0.5, -1.5)))) * 0.5, 0., 0., 1.0);
}