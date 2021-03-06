#version 450

layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

layout(set = 0, binding = 0, rgba8) uniform writeonly image2D img;

struct CSGCommand {
    uint cmd_type;
    uint param_offset;
};

layout(set = 0, binding = 1) buffer CSGCommandBuffer {
    CSGCommand data[];
} csg_commands;

layout(set = 0, binding = 2) buffer CSGParamBuffer {
    uint data[];
} csg_params;

layout(push_constant) uniform PushConstants {
    float min_dist;
    float max_dist;
    uint cmd_count;
    float t;
} push_constants;

// Execution context
#define VALUE_STACK_MAX_SIZE 32

float value_stack_data[VALUE_STACK_MAX_SIZE];
uint value_stack_size;

// CSG Commands
#define CMD_TYPE_SPHERE 0
#define CMD_TYPE_UNION 100
#define CMD_TYPE_SUBTRACTION 101
#include "./csg/primitives/mod.glsl"
#include "./csg/operations/mod.glsl"

/// Computes distance from point `p` to the scene.
float map_scene(in vec3 p) {
    // Early return for empty scenes
    if (push_constants.cmd_count == 0) {
        return push_constants.max_dist;
    }

    // Reset stack
    value_stack_size = 0;

    for (uint cmd_index = 0; cmd_index < push_constants.cmd_count; ++cmd_index) {
        // Get the next command
        CSGCommand cmd = csg_commands.data[cmd_index];

        switch (cmd.cmd_type) {
            case CMD_TYPE_SPHERE: {
                cmd_sphere(p, cmd.param_offset);
                break;
            }
            case CMD_TYPE_UNION: {
                cmd_union();
                break;
            }
            case CMD_TYPE_SUBTRACTION: {
                cmd_subtract();
                break;
            }
        }
    }

//    // Should be unreachable
//    if (value_stack_size == 0) {
//        return push_constants.max_dist;
//    }

    // Pop last value of the stack
    return value_stack_data[--value_stack_size];
}

/// Calculate the normal vector at point `p`.
/// See: https://www.iquilezles.org/www/articles/normalsSDF/normalsSDF.htm
vec3 calculate_normal(in vec3 p) {
    // Tetrahedron technique
    const float eps = 0.0001;
    const vec2 k = vec2(1, -1);
    return normalize(
        k.xyy * map_scene(p + k.xyy * eps) +
        k.yyx * map_scene(p + k.yyx * eps) +
        k.yxy * map_scene(p + k.yxy * eps) +
        k.xxx * map_scene(p + k.xxx * eps)
    );

    // Central differences
//    const float eps = 0.0001;
//    const vec3 h = vec3(eps, 0.0, 0.0);
//    vec3 normal = vec3(
//        map_scene(p + h.xyy) - map_scene(p - h.xyy),
//        map_scene(p + h.yxy) - map_scene(p - h.yxy),
//        map_scene(p + h.yyx) - map_scene(p - h.yyx)
//    );
//    return normalize(normal);
}

/// March a ray through the scene, starting at the ray origin `ro` in direction `rd`.
vec3 ray_march(in vec3 ro, in vec3 rd) {
    const uint NUMBER_OF_STEPS = 64;

    // Ray-march the scene
    float ray_dist = 0.0;

    for (uint i = 0; i < NUMBER_OF_STEPS; ++i) {
        // Current position along the ray
        vec3 p = ro + ray_dist * rd;

        float scene_dist = map_scene(p);

        if (scene_dist < push_constants.min_dist) {
//            return vec3(1.0, 0.0, 0.0);

            vec3 normal = calculate_normal(p);

            vec3 light_position = vec3(2.0, -5.0, 3.0);
            vec3 direction_to_light = normalize(p - light_position);

            float diffuse_intensity = max(0.02, dot(normal, direction_to_light));

            return vec3(0.4, 0.7, 0.1) * diffuse_intensity;
        }

        if (scene_dist > push_constants.max_dist) {
            break;
        }

        ray_dist += scene_dist;
    }

    // Ray-trace the floor plane
    // TODO: Add CSG node for this?
    float tp1 = (-1.5 - ro.y) / rd.y;
    if (tp1 > 0.0) {
        // Basic checkerboard pattern
        // See: https://iquilezles.org/www/articles/checkerfiltering/checkerfiltering.htm
        vec3 p = ro + rd * tp1;
        ivec2 ip = ivec2(round(p.xz+.5));
        float col = float((ip.x^ip.y)&1);
        return vec3(0.1, 0.1, 0.2) + vec3(0.2) * col;
    }

    return vec3(0.0);
}

#define AA 3

void main() {
    vec2 img_dims = vec2(imageSize(img));

    // Camera
    float angle = push_constants.t * 0.5;
    vec3 camera_position = vec3(5.0 * cos(angle), 2.0, 5.0 * sin(angle));
    vec3 camera_up = vec3(0.0, 1.0, 0.0);
    vec3 camera_target = vec3(0.0);

    float focal_length = 2.5;

    // Camera matrix
    vec3 ww = normalize(camera_target - camera_position);
    vec3 uu = normalize(cross(ww, camera_up));
    vec3 vv = normalize(cross(uu, ww));

    // Ray origin
    vec3 ro = camera_position;

    vec3 total_color = vec3(0.0);

    for (uint m = 0; m < AA; ++m) {
        for (uint n = 0; n < AA; ++n) {
            // Pixel offset for anti-aliasing
            vec2 aa_offset = vec2(float(m), float(n)) / float(AA) - 0.5;
            vec2 uv = (2.0 * (gl_GlobalInvocationID.xy + aa_offset) - img_dims.xy) / img_dims.y;

            // Ray direction
            vec3 rd = normalize(vec3(uv.x * uu + uv.y * vv + focal_length * ww));

            // Ray march
            vec3 color = ray_march(ro, rd);

            // Gamma
            color = sqrt(color);
            total_color += color;
        }
    }

    total_color = total_color / float(AA * AA);

    imageStore(img, ivec2(gl_GlobalInvocationID.xy), vec4(total_color, 1.0));
}
