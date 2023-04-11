const SIZE = 100u;
const SIZEI32 = 100;


struct Instance {
    @location(5) pos: vec3<f32>,
    @location(6) state: u32
}

struct Rule {
    survive_mask: u32,
    born_mask: u32,
    max_state: u32,
    neighborhood: u32,
}

@group(0) @binding(0)
var<uniform> rule: Rule;

@group(0) @binding(1)
var<storage,read> cells_in: array<u32>;

@group(0) @binding(2)
var<storage,read_write> cells_out: array<u32>;

@group(0) @binding(3)
var<storage, read_write> instances: array<Instance>;

@compute 
@workgroup_size(1)
fn cs_main(@builtin(global_invocation_id) index: vec3<u32>) {
    let flat_index = flatten_index(index);
    let count = count_neighbors(index);

    let current = cells_in[flat_index];
    if current == 1u && survive(count) {
        cells_out[flat_index] = current;
        instances[flat_index].state = current;
        } else if current == 0u && born(count) {
            cells_out[flat_index] = rule.max_state;
            instances[flat_index].state = rule.max_state;
        } else if current >= 1u{
            cells_out[flat_index] = (current - 1u);
            instances[flat_index].state = (current - 1u);
        } else {
            cells_out[flat_index] =  0u;
            instances[flat_index].state = 0u;
        }

}



fn count_neighbors(index: vec3<u32>) -> u32 {
    var count: u32;
    switch rule.neighborhood {
        case 0u: {count = moore_neighborhood(index);}
        case 1u: {count = moore_neighborhood_non_wrapping(index);}
        case 2u: {count = von_neumann_neigborhood(index);}
        case 3u: {count = von_neumann_neigborhood_non_wrapping(index);}
        default: {count = moore_neighborhood(index);}
    }
    return count;
}

fn moore_neighborhood(index: vec3<u32>) -> u32 {
    var sum = 0u;
    for (var x = -1; x <= 1; x++) {
        for (var y = -1; y <= 1; y++) {
            for (var z = -1; z <= 1; z++) {
                let new_index = wrap_index(vec3<i32>(i32(index.x) + x,i32(index.y) + y,i32(index.z) + z));
                let flat_index = flatten_index(new_index);
                if cells_in[flat_index] == rule.max_state && any(new_index != index) {
                    sum ++;
                }
            }
        }
    }
    return sum;
}

//TODO
fn moore_neighborhood_non_wrapping(index: vec3<u32>) -> u32 {
    var sum = 0u;
    for (var x = -1; x <= 1; x++) {
        for (var y = -1; y <= 1; y++) {
            for (var z = -1; z <= 1; z++) {
                let new_index = wrap_index(vec3<i32>(i32(index.x) + x,i32(index.y) + y,i32(index.z) + z));
                let flat_index = flatten_index(new_index);
                if cells_in[flat_index] == rule.max_state && any(new_index != index) {
                    sum ++;
                }
            }
        }
    }
    return sum;
}
fn von_neumann_neigborhood(index: vec3<u32>) -> u32 {
    var sum = 0u;
    for (var x = -1; x <= 1; x++) {
        for (var y = -1; y <= 1; y++) {
            for (var z = -1; z <= 1; z++) {
                let new_index = wrap_index(vec3<i32>(i32(index.x) + x,i32(index.y) + y,i32(index.z) + z));
                let flat_index = flatten_index(new_index);
                if cells_in[flat_index] == rule.max_state && any(new_index != index) {
                    sum ++;
                }
            }
        }
    }
    return sum;
}
fn von_neumann_neigborhood_non_wrapping(index: vec3<u32>) -> u32 {
    var sum = 0u;
    for (var x = -1; x <= 1; x++) {
        for (var y = -1; y <= 1; y++) {
            for (var z = -1; z <= 1; z++) {
                let new_index = wrap_index(vec3<i32>(i32(index.x) + x,i32(index.y) + y,i32(index.z) + z));
                let flat_index = flatten_index(new_index);
                if cells_in[flat_index] == rule.max_state && any(new_index != index) {
                    sum ++;
                }
            }
        }
    }
    return sum;
}

fn wrap_index(idx: vec3<i32>) -> vec3<u32>{
    return vec3<u32>(u32((idx.x + SIZEI32) % SIZEI32),u32((idx.y + SIZEI32) % SIZEI32) ,u32((idx.z + SIZEI32) % SIZEI32));
}
fn flatten_index(idx: vec3<u32>) -> u32{
    return idx.x * SIZE * SIZE + idx.y * SIZE + idx.z;
}

fn survive(count: u32) -> bool {
    return (rule.survive_mask & (1u << count)) != 0u;
}
fn born(count: u32) -> bool {
    return (rule.born_mask & (1u << count)) != 0u;
}