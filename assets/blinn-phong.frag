out vec4 out_color;

in vec3 frag_pos;
in vec2 frag_uv;
in vec3 frag_normal;
in vec3 frag_eye_normal;

uniform sampler2D u_texture;
const vec3 light_dir = vec3(0.1, 1.0, -0.5);
const float ambient_factor = 0.1;
const float specular_factor = 0.1;
const float diffuse_factor = 1.0 - specular_factor - ambient_factor;


const int specular_power = 2;

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

  float s = pow(max(dot(view_dir, reflect_dir), 0.0),
                specular_power);

  vec3 specular = s * specular_color;
  vec3 diffuse = d * albedo.xyz * diffuse_factor;
  vec3 ambient = albedo.xyz * ambient_factor;
  out_color = vec4(ambient + diffuse + specular, albedo.w);
}