struct Sphere {
    vec3 center;
    float radius;
};

float sdf_sphere(vec3 p, Sphere s) {
    return length(s.center - p) - s.radius;
}
