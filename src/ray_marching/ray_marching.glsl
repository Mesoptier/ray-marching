#version 450

#include "./primitives/sphere/sdf.glsl"

layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

layout(set = 0, binding = 0, rgba8) uniform writeonly image2D img;

layout(set = 0, binding = 1) buffer SceneBuffer {
    uint data[];
} scene_buffer;

layout(push_constant) uniform PushConstants {
    float min_dist;
    float max_dist;
    uint scene_size;
} push_constants;

float sdf_scene(vec3 p) {
    float dist = push_constants.max_dist;

    for (uint i = 0; i < push_constants.scene_size;) {
//        uint type = scene_buffer[i++];
//
//        switch (type) {
//            case 0: // deserialize sphere + call sphere_sdf(p, sphere)
//        }

        Sphere sphere = {
            vec3(
                uintBitsToFloat(scene_buffer.data[i++]),
                uintBitsToFloat(scene_buffer.data[i++]),
                uintBitsToFloat(scene_buffer.data[i++])
            ),
            uintBitsToFloat(scene_buffer.data[i++])
        };

        dist = min(dist, sdf_sphere(p, sphere));

        if (dist < push_constants.min_dist) {
            break;
        }
    }

    return dist;
}

void main() {
    vec4 write_color = vec4(sdf_scene(vec3(gl_GlobalInvocationID.xy / 100.0, 0.0)), 0.0, 0.0, 1.0);
    imageStore(img, ivec2(gl_GlobalInvocationID.xy), write_color);
}
