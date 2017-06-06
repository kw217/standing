// Vertex shader for standing wave simulator

#version 150 core

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

in float a_X;
in float a_P;
in float a_Q;
in float a_NextP;
in float a_NextQ;

flat out vec4 v_Colour;
// unit normal of triangle from here to forward tangent to next cross-section vertex.
flat out vec3 v_Normal;

void main() {
    v_Colour = a_Colour;
    vec3 base = vec3(a_X, a_Ampl * sin((a_X * a_Freq) + a_Phase), 0.0);
    // tangent of the wave itself
    vec3 tangent = vec3(1.0, a_Ampl * a_Freq * cos((a_X * a_Freq) + a_Phase), 0.0);

    vec3 pv = a_P * a_PV;
    vec3 qv = a_Q * a_QV;
    vec3 nextPv = a_NextP * a_PV;
    vec3 nextQv = a_NextQ * a_QV;
    vec3 pos = base + pv + qv;
    vec3 nextPos = base + nextPv + nextQv;

    // Three points of a triangle: POS - TANGENT - NEXT
    vec4 modelPos4 = u_Model * vec4(pos, 1.0);
    vec4 modelTangent4 = u_Model * vec4(pos + tangent, 1.0);
    vec4 modelNext4 = u_Model * vec4(nextPos, 1.0);

    vec3 modelPos = modelPos4.xyz / modelPos4.w;
    vec3 modelTangent = modelTangent4.xyz / modelTangent4.w;
    vec3 modelNext = modelNext4.xyz / modelNext4.w;

    vec3 triPT = modelTangent - modelPos;
    vec3 triPN = modelNext - modelPos;

    v_Normal = normalize(cross(triPT, triPN));

    gl_Position = u_View * vec4(modelPos, 1.0);
}
