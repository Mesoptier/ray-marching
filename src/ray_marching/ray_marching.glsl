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
    uint node_count;
    float t;
} push_constants;

// Execution context
#define VALUE_STACK_MAX_SIZE 32

float value_stack_data[VALUE_STACK_MAX_SIZE];
uint value_stack_size;

// CSG Commands
#define NODE_TYPE_SPHERE 0
#define NODE_TYPE_UNION 100
#define NODE_TYPE_SUBTRACTION 101
#include "./csg/primitives/mod.glsl"
#include "./csg/operations/mod.glsl"

float sdf_scene(in vec3 p) {
    // Early return for empty scenes
    if (push_constants.node_count == 0) {
        return push_constants.max_dist;
    }

    // Reset stack
    value_stack_size = 0;

    for (uint cmd_index = 0; cmd_index < push_constants.node_count; ++cmd_index) {
        // Get the next command
        CSGCommand cmd = csg_commands.data[cmd_index];

        switch (cmd.cmd_type) {
            case NODE_TYPE_SPHERE: {
                cmd_sphere(p, cmd.param_offset);
                break;
            }
            case NODE_TYPE_UNION: {
                cmd_union();
                break;
            }
            case NODE_TYPE_SUBTRACTION: {
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
vec3 calculate_normal(in vec3 p) {
    const vec3 h = vec3(0.001, 0.0, 0.0);

    vec3 normal = vec3(
        sdf_scene(p + h.xyy) - sdf_scene(p - h.xyy),
        sdf_scene(p + h.yxy) - sdf_scene(p - h.yxy),
        sdf_scene(p + h.yyx) - sdf_scene(p - h.yyx)
    );

    return normalize(normal);
}

/// March a ray through the scene, starting at the ray origin `ro` in direction `rd`.
vec3 ray_march(in vec3 ro, in vec3 rd) {
    const uint NUMBER_OF_STEPS = 32;

    float ray_dist = 0.0;

    for (uint i = 0; i < NUMBER_OF_STEPS; ++i) {
        // Current position along the ray
        vec3 p = ro + ray_dist * rd;

        float scene_dist = sdf_scene(p);

        if (scene_dist < push_constants.min_dist) {
//            return vec3(1.0, 0.0, 0.0);

            vec3 normal = calculate_normal(p);

            vec3 light_position = vec3(2.0, -5.0, 3.0);
            vec3 direction_to_light = normalize(p - light_position);

            float diffuse_intensity = max(0.0, dot(normal, direction_to_light));

            return vec3(sin(push_constants.t), -sin(push_constants.t), 0.0) * diffuse_intensity;
        }

        if (scene_dist > push_constants.max_dist) {
            break;
        }

        ray_dist += scene_dist;
    }

    return vec3(0.0);
}

void main() {
    vec2 img_dims = vec2(imageSize(img));
    vec2 uv = vec2(
        (gl_GlobalInvocationID.x - img_dims.x / 2.0) / img_dims.x * 2.0,
        (gl_GlobalInvocationID.y - img_dims.y / 2.0) / img_dims.x  * 2.0
    );

    vec3 camera_position = vec3(0.0, 0.0, -5.0);

    vec3 ro = camera_position;
    vec3 rd = vec3(uv, 1.0);

    vec4 write_color = vec4(ray_march(ro, rd), 1.0);
    imageStore(img, ivec2(gl_GlobalInvocationID.xy), write_color);
}
