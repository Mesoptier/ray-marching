#version 450

#include "./csg/primitives/sphere/sdf.glsl"

layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

layout(set = 0, binding = 0, rgba8) uniform writeonly image2D img;

struct CSGNode {
    uint node_type;
    uint param_offset;
    uint child_count;
};

layout(set = 0, binding = 1) buffer CSGNodeBuffer {
    CSGNode data[];
} csg_nodes;

layout(set = 0, binding = 2) buffer CSGParamBuffer {
    uint data[];
} csg_params;

layout(push_constant) uniform PushConstants {
    float min_dist;
    float max_dist;
    uint node_count;
    float t;
} push_constants;

struct Call {
    CSGNode node;
    uint processed_child_count;
};

#define VALUE_STACK_MAX_SIZE 32
#define CALL_STACK_MAX_SIZE 32

float sdf_scene(in vec3 p) {
    float value_stack[VALUE_STACK_MAX_SIZE];
    uint value_stack_size = 0;

    Call call_stack[CALL_STACK_MAX_SIZE];
    uint call_stack_size = 0;

    for (uint node_index = 0; node_index < push_constants.node_count; ++node_index) {
        // Get the next node
        CSGNode node = csg_nodes.data[node_index];

        // Push this call onto the call stack
        call_stack[call_stack_size] = Call(node, 0);
        ++call_stack_size;

        // Start call
        switch (node.node_type) {
            // Sphere
            case 0: break;

            // Union
            case 100: break;
        }

        // Unwind call stack
        while (call_stack_size > 0 && call_stack[call_stack_size - 1].node.child_count == call_stack[call_stack_size - 1].processed_child_count) {
            float value = 0.0;

            // Finish call
            switch (call_stack[call_stack_size - 1].node.node_type) {
                // Sphere
                case 0: {
                    uint param_offset = call_stack[call_stack_size - 1].node.param_offset;
                    Sphere sphere = {
                        vec3(
                            uintBitsToFloat(csg_params.data[param_offset + 0]),
                            uintBitsToFloat(csg_params.data[param_offset + 1]),
                            uintBitsToFloat(csg_params.data[param_offset + 2])
                        ),
                        uintBitsToFloat(csg_params.data[param_offset + 3])
                    };

                    value = sdf_sphere(p, sphere);
                    break;
                }

                // Union
                case 100: {
                    float v1 = value_stack[--value_stack_size];
                    float v2 = value_stack[--value_stack_size];
                    value = min(v1, v2);
                    break;
                }
            }

            // Push return value onto the value stack
            value_stack[value_stack_size] = value;
            ++value_stack_size;

            // Pop completed call from the stack
            --call_stack_size;

            if (call_stack_size > 0) {
                ++call_stack[call_stack_size - 1].processed_child_count;
            }
        }
    }

    if (value_stack_size == 0) {
        return push_constants.max_dist;
    }

    return value_stack[0];
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
