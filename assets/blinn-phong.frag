out vec4 out_color;

in vec3 frag_pos;
in vec2 frag_uv;
in vec3 frag_normal;
in vec3 frag_eye_normal;

uniform sampler2D u_texture;
const vec3 light_dir = vec3(0.1, 1.0, -0.5);
const float ambient_factor = 0.1;
const float specular_factor = 0.3;
const float diffuse_factor = 1.0  - ambient_factor;

const int shininess = 255;

const vec3 specular_color = vec3(1.0, 1.0, 1.0) * specular_factor;

float
diffuse()
{
  float d = dot(light_dir, frag_normal);
  d = max(d, 0.0);
  return d;
}

void
main()
{

  vec4 albedo = texture(u_texture, frag_uv);
  float d = diffuse();



  vec3 view_dir = normalize(-frag_pos);
  vec3 reflect_dir = reflect(-light_dir, normalize(frag_eye_normal));

  vec3 n = normalize(frag_eye_normal);
  vec3 e = normalize(vec3(frag_pos));
  vec3 specular = vec3(0.0);
  float intensity = max(dot(n, light_dir), 0.0);
  if (intensity > 0.0){
    vec3 half_vector = normalize(light_dir - e);
    float spec_factor = max(dot(half_vector, n), 0.0);
    specular = specular_color * pow(spec_factor, shininess);
    // specular = normalize(half_vector);
  }

  vec3 diffuse = d * albedo.xyz * diffuse_factor;
  vec3 ambient = albedo.xyz * ambient_factor;
  out_color = vec4(specular + diffuse + ambient, albedo.w);
}