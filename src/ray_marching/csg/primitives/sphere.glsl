struct Sphere {
    vec3 center;
    float radius;
};

float sdf_sphere(vec3 p, Sphere s) {
    return length(s.center - p) - s.radius;
}

void cmd_sphere(vec3 p, uint param_offset) {
    Sphere sphere = {
        vec3(
            uintBitsToFloat(csg_params.data[param_offset + 0]),
            uintBitsToFloat(csg_params.data[param_offset + 1]),
            uintBitsToFloat(csg_params.data[param_offset + 2])
        ),
        uintBitsToFloat(csg_params.data[param_offset + 3])
    };

    value_stack_data[value_stack_size++] = sdf_sphere(p, sphere);
}
