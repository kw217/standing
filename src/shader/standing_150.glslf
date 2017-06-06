// Fragment shader for standing wave simulator

#version 150 core
flat in vec4 v_Colour;
flat in vec3 v_Normal;
out vec4 Target0;

uniform Locals {
    mat4 u_Model;
    mat4 u_View;
    vec4 a_Colour;
    vec3 a_PV;
    float a_Phase;
    vec3 a_QV;
    float a_Freq;
    vec3 u_Light;
    float a_Ampl;
};

void main() {
    float brightness = dot(v_Normal, normalize(u_Light));
    vec4 dark_color = vec4(0.1, 0.1, 0.1, v_Colour.a);
    Target0 = mix(dark_color, v_Colour, brightness);
}
