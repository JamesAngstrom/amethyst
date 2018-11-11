// TODO: Needs documentation.

#version 330 core

layout (std140) uniform FragmentArgs {
    uint point_light_count;
    uint directional_light_count;
};

struct PointLight {
    vec3 position;
    vec3 color;
    float pad; // Workaround for bug in mac's implementation of opengl (loads garbage when accessing members of structures in arrays with dynamic indices).
    float intensity;
};

layout (std140) uniform PointLights {
    PointLight plight[128];
};

struct DirectionalLight {
    vec3 color;
    vec3 direction;
};

layout (std140) uniform DirectionalLights {
    DirectionalLight dlight[16];
};

uniform vec3 ambient_color;
uniform vec3 camera_position;

uniform sampler2D albedo_yz;
uniform sampler2D emission_yz;
uniform sampler2D normal_yz;

uniform sampler2D albedo_xz;
uniform sampler2D emission_xz;
uniform sampler2D normal_xz;

uniform sampler2D albedo_xy;
uniform sampler2D emission_xy;
uniform sampler2D normal_xy;

layout (std140) uniform AlbedoOffset {
    vec2 u_offset;
    vec2 v_offset;
} albedo_offset;

layout (std140) uniform EmissionOffset {
    vec2 u_offset;
    vec2 v_offset;
} emission_offset;

layout (std140) uniform NormalOffset {
    vec2 u_offset;
    vec2 v_offset;
} normal_offset;

in VertexData {
    vec3 position;
    vec3 normal;
    vec3 tangent;
    vec2 tex_coord;
} vertex;

out vec4 out_color;

vec3 triplanar_blend(vec3 normal) {  
  vec3 blend = pow(normal, vec3(4.0, 4.0, 4.0));
  blend /= dot(blend, vec3(1.0, 1.0, 1.0));
  return blend;
}

void main() {
    vec3 normal = normalize(vertex.normal);
    vec3 blend = triplanar_blend(normal);

    // Normal maps in tangent space
    vec3 x_tnormal = texture(normal_yz, mod(vertex.position.zy / 4.0, 1.0)).xyz;
    vec3 y_tnormal = texture(normal_xz, mod(vertex.position.xz / 4.0, 1.0)).xyz;
    vec3 z_tnormal = texture(normal_xy, mod(vertex.position.xy / 4.0, 1.0)).xyz;

    // Whiteout blend
    vec2 x_swiz = x_tnormal.xy + normal.zy;
    x_tnormal = vec3(x_swiz.x, x_swiz.y, abs(x_tnormal.z) * normal.x);
    vec2 y_swiz = y_tnormal.xy + normal.xz;
    y_tnormal = vec3(y_swiz.x, y_swiz.y, abs(y_tnormal.z) * normal.y);
    vec2 z_swiz = z_tnormal.xy + normal.xy;
    z_tnormal = vec3(z_swiz.x, z_swiz.y, abs(z_tnormal.z) * normal.z);
    normal = normalize(x_tnormal.zyx * blend.x + y_tnormal.xzy * blend.y + z_tnormal.xyz * blend.z);

    vec4 x_color = texture(albedo_yz, mod(vertex.position.zy / 4.0, 1.0));
    vec4 y_color = texture(albedo_xz, mod(vertex.position.xz / 4.0, 1.0));
    vec4 z_color = texture(albedo_xy, mod(vertex.position.xy / 4.0, 1.0));

    vec4 color = x_color * blend.x  + y_color * blend.y + z_color * blend.z; // vec4(blend.x, blend.y, blend.z, 0.0);
    //vec4 color = x_color * blend.x  + noise_color * blend.y + z_color * blend.z; // vec4(blend.x, blend.y, blend.z, 0.0);

    // vec4 ecolor = texture(emission, tex_coords(vertex.tex_coord, emission_offset.u_offset, emission_offset.v_offset));
    vec3 lighting = vec3(0.0);
    for (uint i = 0u; i < point_light_count; i++) {
        // Calculate diffuse light
        vec3 light_dir = normalize(plight[i].position - vertex.position);
        float diff = max(dot(light_dir, normal), 0.0);
        vec3 diffuse = diff * normalize(plight[i].color);
        // Calculate attenuation
        vec3 dist = plight[i].position - vertex.position;
        float dist2 = dot(dist, dist);
        float attenuation = (plight[i].intensity / dist2);
        lighting += diffuse * attenuation;
    }
    for (uint i = 0u; i < directional_light_count; i++) {
        vec3 dir = dlight[i].direction;
        float diff = max(dot(-dir, normal), 0.0);
        vec3 diffuse = diff * dlight[i].color;
        lighting += diffuse;
    }
    lighting += ambient_color;
    out_color = vec4(lighting, 1.0) * color;
}
