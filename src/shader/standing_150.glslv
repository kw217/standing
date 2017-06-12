// Vertex shader for standing wave simulator

#version 150 core

uniform Locals {
    mat4 u_Model;
    mat4 u_View;
    vec4 a_Colour;
    vec3 a_PV;
    float a_Phase1;
    vec3 a_QV;
    float a_Freq1;
    vec3 u_Light;
    float a_Ampl1;
    float a_Ampl2;
    float a_Freq2;
    float a_Phase2;
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

    float ampl1 = a_Ampl1;
    float theta1 = (a_X * a_Freq1) + a_Phase1;
    float d_theta1 = a_Freq1;

    float ampl2 = a_Ampl2;
    float theta2 = (a_X * a_Freq2) + a_Phase2;
    float d_theta2 = a_Freq2;

    float y = ampl1 * sin(theta1) + ampl2 * sin(theta2);
    float dy = ampl1 * d_theta1 * cos(theta1) + ampl2 * d_theta2 * cos(theta2);

    vec3 base = vec3(a_X, y, 0.0);
    // tangent of the wave itself
    vec3 tangent = vec3(1.0, dy, 0.0);

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
