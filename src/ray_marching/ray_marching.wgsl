struct VertexOut {
    @location(0) uv: vec2<f32>,
    @builtin(position) position: vec4<f32>,
}

var<private> v_positions: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(1.0, -1.0),
    vec2<f32>(-1.0, 1.0),
    vec2<f32>(1.0, 1.0),
);

@vertex
fn vs_main(@builtin(vertex_index) v_idx: u32) -> VertexOut {
    var out: VertexOut;
    out.position = vec4<f32>(v_positions[v_idx], 0.0, 1.0);
    out.uv = v_positions[v_idx];
    return out;
}

@group(0) @binding(2) var<uniform> viewport: vec2<f32>;

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let uv = in.uv * (viewport / viewport.y);

    // Camera
    let angle: f32 = 0.0;
    let camera_position = vec3<f32>(5.0 * cos(angle), 2.0, 5.0 * sin(angle));
    let camera_up = vec3<f32>(0.0, 1.0, 0.0);
    let camera_target = vec3<f32>(0.0);

    let focal_length: f32 = 2.5;

    // Camera matrix
    let ww = normalize(camera_target - camera_position);
    let uu = normalize(cross(ww, camera_up));
    let vv = normalize(cross(uu, ww));

    // Ray origin
    let ray_origin = camera_position;

    var total_color = vec3<f32>(0.0);

    // TODO: Add antialiasing

    // Ray direction
    let ray_direction = normalize(vec3(uv.x * uu + uv.y * vv + focal_length * ww));

    // Ray march
    var color = ray_march(ray_origin, ray_direction);

    // Gamma
    color = sqrt(color);
    total_color += color;

    return vec4<f32>(total_color, 1.0);
}

struct RayMarchLimits {
    min_dist: f32,
    max_dist: f32,
    max_iter: u32,
}

// TODO: Separate bind groups (see https://toji.dev/webgpu-best-practices/bind-groups.html)
@group(0) @binding(0) var<uniform> ray_march_limits: RayMarchLimits;

fn ray_march(origin: vec3<f32>, direction: vec3<f32>) -> vec3<f32> {
    var dist: f32 = 0.0;

    for (var i = 0u; i < ray_march_limits.max_iter; i++) {
        let pos = origin + direction * dist;

        // Distance to the scene
        let scene_dist = map_scene(pos);

        // Return color if we hit something
        if (scene_dist < ray_march_limits.min_dist) {
            let normal = calculate_normal(pos);

            let light_position = vec3<f32>(2.0, -5.0, 3.0);
            let direction_to_light = normalize(pos - light_position);

            let diffuse_intensity = max(0.02, dot(normal, direction_to_light));

            return vec3<f32>(0.4, 0.7, 0.1) * diffuse_intensity;
        }

        // Abort if ray has gone too far
        if (scene_dist > ray_march_limits.max_dist) {
            break;
        }

        // March ray forward
        dist += scene_dist;
    }

    // Ray-trace the floor plane
    // TODO: Add CSG node for this?
    let floor_y = -1.5;
    let floor_dist = (floor_y - origin.y) / direction.y;
    if (floor_dist > 0.0) {
        // Basic checkerboard pattern
        // See: https://iquilezles.org/www/articles/checkerfiltering/checkerfiltering.htm
        let pos = (origin + direction * floor_dist).xz;
        let ipos = vec2<i32>(round(pos + .5));
        let col = f32((ipos.x ^ ipos.y) & 1);
        return vec3(0.1, 0.1, 0.2) + vec3(0.2) * col;
    }

    return vec3(0.0);
}

/// Calculate the normal vector at point `pos`.
/// See: https://www.iquilezles.org/www/articles/normalsSDF/normalsSDF.htm
fn calculate_normal(pos: vec3<f32>) -> vec3<f32> {
    let eps = 0.0001;
    let k = vec2<f32>(1.0, -1.0);
    return normalize(
        k.xyy * map_scene(pos + k.xyy * eps) +
        k.yyx * map_scene(pos + k.yyx * eps) +
        k.yxy * map_scene(pos + k.yxy * eps) +
        k.xxx * map_scene(pos + k.xxx * eps)
    );
}

struct CSGCommandBuffer {
    cmd_count: u32,
    buffer: array<u32>,
}

@group(0) @binding(1) var<storage, read> csg_commands: CSGCommandBuffer;
var<private> csg_commands_ptr: u32;

fn csg_pop_u32() -> u32 {
    let value = csg_commands.buffer[csg_commands_ptr];
    csg_commands_ptr++;
    return value;
}

fn csg_pop_f32() -> f32 {
    return bitcast<f32>(csg_pop_u32());
}

fn csg_pop_vec3() -> vec3<f32> {
    return vec3<f32>(csg_pop_f32(), csg_pop_f32(), csg_pop_f32());
}

fn csg_pop_command_type() -> u32 {
    return csg_pop_u32();
}

// Execution context
const value_stack_max_size: u32 = 32u;
var<private> value_stack_data: array<f32, value_stack_max_size>;
var<private> value_stack_size: u32;

fn pop_value() -> f32 {
    value_stack_size--;
    return value_stack_data[value_stack_size];
}

fn push_value(value: f32) {
    value_stack_data[value_stack_size] = value;
    value_stack_size++;
}

fn map_scene(pos: vec3<f32>) -> f32 {
    // Early return for empty scenes.
    if (csg_commands.cmd_count == 0u) {
        return ray_march_limits.max_dist;
    }

    // Reset pointers.
    value_stack_size = 0u;
    csg_commands_ptr = 0u;

    for (var idx = 0u; idx < csg_commands.cmd_count; idx++) {
        let cmd_type = csg_pop_command_type();
        push_value(eval_cmd(cmd_type, pos));
    }

    return pop_value();
}

fn eval_cmd(cmd_type: u32, pos: vec3<f32>) -> f32 {
    switch (cmd_type) {
        // Primitives
        case 0u: {
            return eval_cmd_sphere(pos);
        }

        // Binary operations
        case 100u: {
            return eval_cmd_union();
        }
        case 101u: {
            return eval_cmd_subtract();
        }

        default: {
            return 0.0;
        }
    }
}

fn eval_cmd_sphere(pos: vec3<f32>) -> f32 {
    let center = csg_pop_vec3();
    let radius = csg_pop_f32();
    return length(pos - center) - radius;
}

fn eval_cmd_union() -> f32 {
    let b = pop_value();
    let a = pop_value();
    return min(a, b);
}

fn eval_cmd_subtract() -> f32 {
    let b = pop_value();
    let a = pop_value();
    return max(a, -b);
}
