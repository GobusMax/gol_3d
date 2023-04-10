
struct Instance {
    @location(5) pos: vec3<f32>,
    @location(6) state: u32
}
//TODO
@group(0) @binding(1)
var<storage,read> cells_in: array<u32>;

@group(0) @binding(2)
var<storage,read_write> cells_out: array<u32>;

@group(0) @binding(3)
var<storage, read_write> instances: array<Instance>;

@compute 
@workgroup_size(1)
fn cs_main(@builtin(global_invocation_id) index: vec3<u32>) {
    let index_flat = index.x * 100u * 100u + index.y * 100u + index.z;
    let index_i32 = vec3<i32>(i32(index.x), i32(index.y), i32(index.z));

    instances[index_flat].state = 1u - instances[index_flat].state;
}



fn count_neighbors(index_flat: u32) -> u32 {
    let sum = 0u;
    for (var i = -1; i <= 1; i++) {
    }

    return sum;
}