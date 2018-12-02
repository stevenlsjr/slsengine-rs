#define N_LIGHTS 4

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

uniform vec3 ambient_factor = vec3(1.0, 1.0, 1.0) * 0.01;

uniform vec3 light_positions[N_LIGHTS];

const vec3 light_colors[N_LIGHTS] = vec3[](vec3(23.47, 21.31, 20.79) / 5.0,
                                           vec3(23.47, 21.31, 20.79) / 5.0,
                                           vec3(23.47, 21.31, 20.79) / 5.0,
                                           vec3(23.47, 21.31, 20.79) / 5.0);

const float light_constants[N_LIGHTS] = float[](1.0, 1.0, 1.0, 1.0);

const float light_linears[N_LIGHTS] = float[](0.014, 0.014, 0.014, 0.014);

const float light_quadratics[N_LIGHTS] = float[](0.01, 0.01, 0.01, 0.01);

layout(std140) uniform Material
{
  vec4 albedo_factor;     // size 16
  float roughness_factor; // 4
  float metallic_factor;  // 4
  vec3 emissive;          // 16
};                        // min size=48

float
calculate_attenuation(float light_distance, int i)
{

  return 1.0 / (light_constants[i] + light_linears[i] * light_distance +
                light_quadratics[i] * light_distance * light_distance);
}

float
chi_ggx(float x)
{
  return x > 0 ? 1.0 : 0.0;
}

vec3
fresnel_schlick(float cos_theta, vec3 F0)
{
  return F0 + (1.0 - F0) * pow(1.0 - cos_theta, 5.0);
}

float
ggx_distribution(vec3 normal, vec3 H, float roughness)
{
  float alpha = roughness * roughness;
  float n_dot_h = dot(normal, H);
  float alpha_2 = alpha * alpha;
  float n_dot_h_2 = n_dot_h * n_dot_h;
  float denominator = n_dot_h_2 * alpha_2 + (1.0 - n_dot_h_2);

  return (chi_ggx(n_dot_h) * alpha_2) / (PI * denominator * denominator);
}

float
smith_geometry__schlick_ggx(float n_dot_v, float roughness)
{
  float r = roughness + 1.0;
  float k = (r * r) / 8.0;
  return n_dot_v / (n_dot_v * (1.0 - k) + k);
}

float
smith_geometry(vec3 normal, vec3 V, vec3 L, float roughness)
{
  float n_dot_v = max(dot(normal, V), 0.0);
  float n_dot_l = max(dot(normal, L), 0.0);
  float ggx2 = smith_geometry__schlick_ggx(n_dot_v, roughness);
  float ggx1 = smith_geometry__schlick_ggx(n_dot_l, roughness);
  return ggx2 * ggx1;
}

vec3
get_normal()
{
  vec3 N = normalize(frag_eye_normal);
  return N;
}

void
main()
{
  vec4 albedo_texel = texture(u_texture, frag_uv);
  vec3 N = get_normal();
  vec3 V = normalize(CAM_POS - frag_pos);
  float roughness = roughness_factor;
  float metalness = metallic_factor;
  vec3 albedo = albedo_factor.rgb;

  vec3 F0 = vec3(0.04);
  F0 = mix(F0, albedo, metalness);

  vec3 Lo = vec3(0.0);

  vec3 L_outgoing = vec3(0);

  for (int i = 0; i < 4; ++i) {

    vec3 light_position = light_positions[i];
    vec3 light_dir = normalize(light_position - frag_pos);
    vec3 H = normalize(V + light_dir);

    float light_distance = length(light_position - frag_pos);
    float attenuation = calculate_attenuation(light_distance, i);

    vec3 radiance = light_colors[i] * attenuation;
    vec3 fresnel = fresnel_schlick(max(dot(H, V), 0.0), F0);
    float normal_distribution = ggx_distribution(N, H, roughness);
    float geometry = smith_geometry(N, V, light_dir, roughness);

    { // calculate i'th subdivision of brdf
      float n_dot_l = max(dot(N, light_dir), 0.0);

      vec3 specular;

      // vec3 numerator = normal_distribution * geometry * fresnel;
      vec3 numerator = normal_distribution * geometry * fresnel;
      float denominator =
        4.0 * max(dot(N, V), 0.0) * max(dot(N, light_dir), 0.0);
      specular = numerator / max(denominator, 0.0001);

      vec3 k_s = fresnel;
      vec3 k_d = vec3(1.0) - k_s;
      k_d *= 1.0 - metalness;
      L_outgoing += (k_d * albedo / PI + specular) * radiance * n_dot_l;
    }
  }

  vec3 ambient = ambient_factor * albedo;
  vec3 color = ambient + L_outgoing;
  // HDR tonemap
  color = color / (color + vec3(1));
  // gamma correction
  color = pow(color, vec3(1.0 / 2.2));

  out_color = vec4(color, 1.0);
}