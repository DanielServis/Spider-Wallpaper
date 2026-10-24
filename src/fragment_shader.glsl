#version 330 core 

out vec4 FragColor;
in vec3 FragPos;

uniform float time;

void main() {
    float dist = length(FragPos);
    float normalised_dist = dist / 10;
    vec3 near_color = vec3(1.0, 1.0, 1.0);
    vec3 far_color = vec3(1.0, 1.0, 1.0);
    vec3 color = mix(near_color, far_color, normalised_dist);
    FragColor = vec4(color, 1.0);
}
