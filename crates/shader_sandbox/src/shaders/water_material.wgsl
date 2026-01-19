#import bevy_pbr::{
    forward_io::{FragmentOutput, Vertex, VertexOutput},
    mesh_bindings::mesh,
    mesh_functions,
    mesh_view_bindings::{depth_prepass_texture, globals, lights, view, view_transmission_sampler, view_transmission_texture},
    view_transformations::position_world_to_clip,
}
#import bevy_render::view::{
    depth_ndc_to_view_z,
    frag_coord_to_ndc,
    frag_coord_to_uv,
    ndc_to_uv,
    position_view_to_clip,
}

struct WaterMaterialParams {
    surface_color: vec4<f32>,
    volume_color: vec4<f32>,
    rim_color: vec4<f32>,
    rim_strength: f32,
    base_alpha: f32,
    reflection_blend: f32,
    micro_scale: f32,
    micro_strength: f32,
    env_strength: f32,
    spec_strength: f32,
    spec_power: f32,
    clearcoat_strength: f32,
    clearcoat_power: f32,
    sampler1_speed: vec2<f32>,
    sampler2_speed: vec2<f32>,
    sampler_scale: vec2<f32>,
    sampler_mix: f32,
    normal_strength: f32,
    height_strength: f32,
    refraction_amount: f32,
    edge_size: f32,
    air_ior: f32,
    ior: f32,
    far_clip: f32,
    ssr_screen_fade: f32,
    ssr_strength: f32,
    ssr_thickness: f32,
    planar_reflection_strength: f32,
    steps: u32,
    fog_underwater: u32,
    foam_or_fade: u32,
    debug_view: u32,
    lighting_enabled: u32,
    _padding: u32,
    reflection_clip_from_world: mat4x4<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: WaterMaterialParams;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var normal1_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var normal1_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var normal2_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var normal2_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(5) var height1_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(6) var height1_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(7) var height2_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(8) var height2_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(9) var edge1_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(10) var edge1_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(11) var edge2_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(12) var edge2_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(13) var environment_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(14) var environment_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(15) var micro_normal_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(16) var micro_normal_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(17) var reflection_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(18) var reflection_sampler: sampler;

const PI: f32 = 3.141592653589793;

fn schlick_fresnel(ior1: f32, ior2: f32, view_dir: vec3<f32>, normal: vec3<f32>) -> f32 {
    let incident = clamp(dot(view_dir, normal), 0.0, 1.0);
    let reflectance = pow((ior2 - ior1) / (ior2 + ior1), 2.0);
    return reflectance + (1.0 - reflectance) * pow(1.0 - incident, 5.0);
}

fn snells_window(normal: vec3<f32>, view_dir: vec3<f32>, ior: f32) -> f32 {
    let cos_theta = clamp(dot(normal, view_dir), 0.0, 1.0);
    let sin_theta = sqrt(max(0.0, 1.0 - cos_theta * cos_theta));
    return select(0.0, 1.0, sin_theta * ior <= 1.0);
}

fn edge_fade(uv: vec2<f32>, size: f32) -> f32 {
    if (size <= 0.0) {
        return 1.0;
    }
    let x1 = clamp(uv.x / size, 0.0, 1.0);
    let x2 = clamp((1.0 - uv.x) / size, 0.0, 1.0);
    let y1 = clamp(uv.y / size, 0.0, 1.0);
    let y2 = clamp((1.0 - uv.y) / size, 0.0, 1.0);
    return x1 * x2 * y1 * y2;
}

fn direction_to_env_uv(direction: vec3<f32>) -> vec2<f32> {
    let dir = normalize(direction);
    let u = atan2(dir.z, dir.x) / (2.0 * PI) + 0.5;
    let v = acos(clamp(dir.y, -1.0, 1.0)) / PI;
    return vec2<f32>(u, v);
}

fn water_height(world_xz: vec2<f32>) -> f32 {
    let uv1 = world_xz * material.sampler_scale + material.sampler1_speed * globals.time;
    let uv2 = world_xz * material.sampler_scale + material.sampler2_speed * globals.time;
    let h1 = textureSampleLevel(height1_texture, height1_sampler, uv1, 0.0).r;
    let h2 = textureSampleLevel(height2_texture, height2_sampler, uv2, 0.0).r;
    let height = mix(h1, h2, material.sampler_mix);
    return (height - 0.5) * material.height_strength;
}

@vertex
fn vertex(vertex_in: Vertex) -> VertexOutput {
    var out: VertexOutput;
    var vertex = vertex_in;

    let world_from_local = mesh_functions::get_world_from_local(vertex_in.instance_index);

#ifdef VERTEX_POSITIONS
    let world_pos = mesh_functions::mesh_position_local_to_world(
        world_from_local,
        vec4<f32>(vertex.position, 1.0)
    );
    let uv1 = world_pos.xz * material.sampler_scale + material.sampler1_speed * globals.time;
    let uv2 = world_pos.xz * material.sampler_scale + material.sampler2_speed * globals.time;

    let h1 = textureSampleLevel(height1_texture, height1_sampler, uv1, 0.0).r;
    let h2 = textureSampleLevel(height2_texture, height2_sampler, uv2, 0.0).r;
    let height = mix(h1, h2, material.sampler_mix);
    vertex.position.y += (height - 0.5) * material.height_strength;

    out.world_position = mesh_functions::mesh_position_local_to_world(
        world_from_local,
        vec4<f32>(vertex.position, 1.0)
    );
    out.position = position_world_to_clip(out.world_position.xyz);
#endif

#ifdef VERTEX_NORMALS
    out.world_normal = mesh_functions::mesh_normal_local_to_world(
        vertex.normal,
        vertex_in.instance_index
    );
#endif

#ifdef VERTEX_TANGENTS
    out.world_tangent = mesh_functions::mesh_tangent_local_to_world(
        world_from_local,
        vertex.tangent,
        vertex_in.instance_index
    );
#endif

    return out;
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) front_facing: bool,
) -> FragmentOutput {
    var out: FragmentOutput;

    let world_pos = in.world_position.xyz;
    let uv1 = world_pos.xz * material.sampler_scale + material.sampler1_speed * globals.time;
    let uv2 = world_pos.xz * material.sampler_scale + material.sampler2_speed * globals.time;
    let _frag_uv = frag_coord_to_uv(in.position.xy, view.viewport);

    var normal = normalize(in.world_normal);

#ifdef VERTEX_TANGENTS
    let tangent = normalize(in.world_tangent.xyz);
    let bitangent = normalize(cross(normal, tangent) * in.world_tangent.w);
    let n1 = textureSample(normal1_texture, normal1_sampler, uv1).xyz * 2.0 - vec3<f32>(1.0);
    let n2 = textureSample(normal2_texture, normal2_sampler, uv2).xyz * 2.0 - vec3<f32>(1.0);
    var tangent_normal = normalize(mix(n1, n2, material.sampler_mix));
    tangent_normal = normalize(vec3<f32>(
        tangent_normal.xy * material.normal_strength,
        tangent_normal.z
    ));
    if (material.micro_strength > 0.0 && material.micro_scale > 0.0) {
        let micro_uv =
            world_pos.xz * material.micro_scale + material.sampler1_speed * globals.time * 4.0;
        let micro_n =
            textureSample(micro_normal_texture, micro_normal_sampler, micro_uv).xyz * 2.0
            - vec3<f32>(1.0);
        tangent_normal = normalize(tangent_normal + micro_n * material.micro_strength);
    }
    let tbn = mat3x3<f32>(tangent, bitangent, normal);
    normal = normalize(tbn * tangent_normal);
#endif

    let view_dir = normalize(view.world_position.xyz - world_pos);
    let fresnel = schlick_fresnel(material.air_ior, material.ior, view_dir, normal);
    let view_pos = (view.view_from_world * vec4<f32>(world_pos, 1.0)).xyz;
    let view_dir_vs = normalize(-view_pos);

    var depth_factor = 0.0;
    var depth_diff = 0.0;
    var surface_depth = 0.0;
#ifdef DEPTH_PREPASS
    let depth_ndc = textureLoad(depth_prepass_texture, vec2<i32>(in.position.xy), 0);
    let depth_view = depth_ndc_to_view_z(depth_ndc, view.clip_from_view, view.view_from_clip);
    let frag_ndc = frag_coord_to_ndc(in.position, view.viewport);
    let frag_view = depth_ndc_to_view_z(frag_ndc.z, view.clip_from_view, view.view_from_clip);
    surface_depth = -frag_view;
    depth_diff = abs(depth_view - frag_view);
    depth_factor = clamp(1.0 - depth_diff / material.far_clip, 0.0, 1.0);
#endif

    var base_color = material.surface_color.xyz;
    if (!front_facing) {
        base_color = material.volume_color.xyz;
    }

    var alpha = clamp(material.base_alpha, 0.0, 1.0);
#ifdef DEPTH_PREPASS
    if (material.edge_size > 0.0 && depth_diff < material.edge_size) {
        let edge_factor = clamp(depth_diff / material.edge_size, 0.0, 1.0);
        if (material.foam_or_fade == 1u) {
            alpha = edge_factor * alpha;
        } else {
            let e1 = textureSample(edge1_texture, edge1_sampler, uv1).r;
            let e2 = textureSample(edge2_texture, edge2_sampler, uv2).r;
            let edge_tex = mix(e1, e2, material.sampler_mix);
            let foam_mask = step(edge_factor, edge_tex);
            base_color = mix(base_color, vec3<f32>(1.0), foam_mask);
        }
    }
#endif

    let view_normal = normalize((view.view_from_world * vec4<f32>(normal, 0.0)).xyz);
    let refract_offset = view_normal.xy * material.refraction_amount * (0.25 + depth_factor);
    let refract_uv = clamp(_frag_uv + refract_offset, vec2<f32>(0.001), vec2<f32>(0.999));
    let refracted = textureSample(view_transmission_texture, view_transmission_sampler, refract_uv).xyz;
    let absorption_color = mix(material.volume_color.xyz, vec3<f32>(1.0), depth_factor);
    let refracted_color = refracted * absorption_color;

    var final_color = clamp(
        refracted_color + material.surface_color.xyz * fresnel,
        vec3<f32>(0.0),
        vec3<f32>(1.0)
    );

    let camera_underwater = view.world_position.y < water_height(view.world_position.xz);
    if (!front_facing) {
        let window = snells_window(view_normal, view_dir_vs, material.ior);
        if (window > 0.5) {
            var fog_color = vec3<f32>(1.0);
            if (camera_underwater && material.fog_underwater != 0u) {
#ifdef DEPTH_PREPASS
                let fog_mix = clamp(1.0 / max(depth_diff, 0.001), 0.0, 1.0);
                fog_color = mix(material.volume_color.xyz, vec3<f32>(1.0), fog_mix);
#else
                fog_color = material.volume_color.xyz;
#endif
            }
            final_color = fog_color * refracted;
        } else {
            final_color = mix(material.volume_color.xyz, base_color, fresnel);
        }
    }

    if (material.env_strength > 0.0) {
        let env_dir = reflect(-view_dir, normal);
        let env_uv = direction_to_env_uv(env_dir);
        let env_color = textureSample(environment_texture, environment_sampler, env_uv).xyz;
        let env_mix = clamp(material.env_strength * material.reflection_blend * fresnel, 0.0, 1.0);
        final_color = mix(final_color, env_color, env_mix);
    }
    if (material.planar_reflection_strength > 0.0) {
        let clip_pos = material.reflection_clip_from_world * vec4<f32>(world_pos, 1.0);
        if (clip_pos.w > 0.0) {
            let ndc = clip_pos.xyz / clip_pos.w;
            let uv = ndc_to_uv(ndc.xy);
            if (uv.x >= 0.0 && uv.x <= 1.0 && uv.y >= 0.0 && uv.y <= 1.0) {
                let reflection_color = textureSampleLevel(
                    reflection_texture,
                    reflection_sampler,
                    uv,
                    0.0
                ).xyz;
                let reflection_mix = clamp(
                    material.planar_reflection_strength * fresnel,
                    0.0,
                    1.0
                );
                final_color = mix(final_color, reflection_color, reflection_mix);
            }
        }
    }

    if (material.lighting_enabled != 0u && lights.n_directional_lights > 0u) {
        let light = lights.directional_lights[0];
        let L = normalize(light.direction_to_light);
        let light_color = clamp(light.color.rgb, vec3<f32>(0.0), vec3<f32>(1.6));
        let ndotl = max(dot(normal, L), 0.0);
        let H = normalize(L + view_dir);
        let spec = pow(max(dot(normal, H), 0.0), material.spec_power)
            * material.spec_strength
            * ndotl
            * fresnel;
        let clearcoat = pow(max(dot(normal, H), 0.0), material.clearcoat_power)
            * material.clearcoat_strength
            * ndotl
            * fresnel;
        final_color = clamp(
            final_color + light_color * (spec + clearcoat),
            vec3<f32>(0.0),
            vec3<f32>(1.0)
        );
    }
    if (material.rim_strength > 0.0) {
        let rim = pow(1.0 - clamp(dot(normal, view_dir), 0.0, 1.0), 2.0);
        final_color = clamp(
            mix(final_color, material.rim_color.xyz, rim * material.rim_strength),
            vec3<f32>(0.0),
            vec3<f32>(1.0)
        );
    }

    var ssr_hit = false;
    var ssr_color = vec3<f32>(0.0);
    var ssr_weight = 0.0;
#ifdef DEPTH_PREPASS
    if (material.steps > 0u) {
        let reflect_dir = reflect(-view_dir_vs, view_normal);
        var hit_uv = vec2<f32>(0.0);
        var hit = false;
        var i: u32 = 0u;

        loop {
            if (i >= material.steps) {
                break;
            }
            let step_scale = f32(i + 1u) / f32(material.steps);
            let step_dist = step_scale * step_scale * material.far_clip;
            let sample_pos = view_pos + reflect_dir * step_dist;
            let clip_pos = position_view_to_clip(sample_pos, view.clip_from_view);
            if (clip_pos.w <= 0.0) {
                break;
            }
            let ndc = clip_pos.xyz / clip_pos.w;
            let uv = ndc_to_uv(ndc.xy);
            if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
                break;
            }

            let pixel = vec2<i32>(view.viewport.xy + uv * view.viewport.zw);
            let sample_depth_ndc = textureLoad(depth_prepass_texture, pixel, 0);
            let sample_depth_view = depth_ndc_to_view_z(
                sample_depth_ndc,
                view.clip_from_view,
                view.view_from_clip
            );
            let ray_depth = -sample_pos.z;
            let scene_depth = -sample_depth_view;

            let depth_delta = ray_depth - scene_depth;
            if (depth_delta >= 0.0 && depth_delta <= material.ssr_thickness) {
                if (abs(scene_depth - surface_depth) <= 0.001) {
                    i += 1u;
                    continue;
                }
                hit_uv = uv;
                hit = true;
                break;
            }
            i += 1u;
        }

        if (hit) {
            ssr_color = textureSampleLevel(
                view_transmission_texture,
                view_transmission_sampler,
                hit_uv,
                0.0
            ).xyz;
            ssr_weight = clamp(edge_fade(hit_uv, material.ssr_screen_fade), 0.0, 1.0);
            let ssr_fresnel = max(
                schlick_fresnel(1.0, material.ior, view_dir, normal),
                0.2
            );
            let ssr_mix = clamp(
                material.reflection_blend * ssr_weight * material.ssr_strength * ssr_fresnel,
                0.0,
                1.0
            );
            final_color = mix(final_color, ssr_color, ssr_mix);
            alpha = max(alpha, ssr_mix);
            ssr_hit = true;
        }
    }
#endif

    if (material.debug_view == 1u) {
        final_color = select(vec3<f32>(0.0), vec3<f32>(0.0, 1.0, 0.0), ssr_hit);
        alpha = 1.0;
    } else if (material.debug_view == 2u) {
        final_color = ssr_color * ssr_weight;
        alpha = 1.0;
    }

    out.color = vec4<f32>(final_color, alpha);
    return out;
}
