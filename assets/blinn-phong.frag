#define N_LIGHTS 1

const float PI = 3.14159265359;

/// calculate lighting from camera point of view
const vec3 CAM_POS = vec3(0.0);

out vec4 out_color;

in vec3 frag_pos;
in vec2 frag_uv;
in vec3 frag_normal;
in vec3 frag_eye_normal;

uniform sampler2D u_texture;
uniform mat4 modelview;


uniform vec3 light_positions[N_LIGHTS];

const vec3 light_colors[N_LIGHTS] = vec3[](vec3(23.47, 21.31, 20.79));

const float light_constants[N_LIGHTS] = float[](1.0);

const float light_linears[N_LIGHTS] = float[](0.09);

const float light_quadratics[N_LIGHTS] = float[](0.032);

layout(std140) uniform Material
{
  vec4 albedo_factor;     // size 16
  float roughness_factor; // 4
  float metallic_factor;  // 4
  vec3 emissive;          // 16
};                        // min size=48

float
calculate_attenuation(vec3 pos, int light_index)
{
  return 0.0;
}

void
main()
{
  vec3 N = normalize(frag_eye_normal);
  vec3 V = normalize(CAM_POS - frag_pos);

  vec3 F0 = vec3(0.04);
  F0 = mix(F0, albedo_factor.xyz, metallic_factor);

  vec3 Lo = vec3(0.0);

  float att = 0.0;
  for (int i = 0; i < N_LIGHTS; ++i) {
    vec3 light_position = light_positions[i];
    vec3 L = normalize(light_position - frag_pos);
    vec3 H = normalize(V + L);
    float distance = length(light_position - frag_pos);
    float attenuation = 1 / (distance * distance);
    att = attenuation;
  }

  out_color = vec4(att * 10, 0.0, 0.0, 0.0);
}