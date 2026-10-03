#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{
        apply_pbr_lighting,
        main_pass_post_lighting_processing,
    }
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
}
#endif

struct MeshPreviewMaps {
    base_color: vec4<f32>,
    normal: vec4<f32>,
    height: vec4<f32>,
    ambient_occlusion: vec4<f32>,
    roughness: vec4<f32>,
    metallic: vec4<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(100)
var base_color_texture: texture_2d<f32>;

@group(#{MATERIAL_BIND_GROUP}) @binding(101)
var base_color_sampler: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(102)
var normal_texture: texture_2d<f32>;

@group(#{MATERIAL_BIND_GROUP}) @binding(103)
var normal_sampler: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(104)
var height_texture: texture_2d<f32>;

@group(#{MATERIAL_BIND_GROUP}) @binding(105)
var height_sampler: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(106)
var ambient_occlusion_texture: texture_2d<f32>;

@group(#{MATERIAL_BIND_GROUP}) @binding(107)
var ambient_occlusion_sampler: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(108)
var roughness_texture: texture_2d<f32>;

@group(#{MATERIAL_BIND_GROUP}) @binding(109)
var roughness_sampler: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(110)
var metallic_texture: texture_2d<f32>;

@group(#{MATERIAL_BIND_GROUP}) @binding(111)
var metallic_sampler: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(112)
var<uniform> maps: MeshPreviewMaps;


@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    // Base Color
    if maps.base_color.x > 0.5 {
        pbr_input.material.base_color =
            textureSample(
                base_color_texture,
                base_color_sampler,
                in.uv,
            );
    }

    // Normal
    if maps.normal.x > 0.5 {
        let tangent_normal =
            textureSample(
                normal_texture,
                normal_sampler,
                in.uv,
            ).rgb * 2.0 - 1.0;

        let N = normalize(in.world_normal);
        let T = normalize(in.world_tangent.xyz);
        let B = normalize(cross(N, T) * in.world_tangent.w);

        let TBN = mat3x3<f32>(T, B, N);

        pbr_input.N = normalize(TBN * tangent_normal);
    }

    // Roughness
    if maps.roughness.x > 0.5 {
        pbr_input.material.perceptual_roughness =
            textureSample(
                roughness_texture,
                roughness_sampler,
                in.uv,
            ).r;
    }

    // Metallic
    if maps.metallic.x > 0.5 {
        pbr_input.material.metallic =
            textureSample(
                metallic_texture,
                metallic_sampler,
                in.uv,
            ).r;
    }

    // Ambient Occlusion
    if maps.ambient_occlusion.x > 0.5 {
        let ao =
            textureSample(
                ambient_occlusion_texture,
                ambient_occlusion_sampler,
                in.uv,
            ).r;

        pbr_input.diffuse_occlusion = vec3<f32>(ao);
    }

    // Height is currently not used directly by the PBR shader.
    // It is available as an atlas map for future parallax/displacement support.

#ifdef PREPASS_PIPELINE

    return deferred_output(in, pbr_input);

#else

    var out: FragmentOutput;

    out.color = apply_pbr_lighting(pbr_input);

    out.color = main_pass_post_lighting_processing(
        pbr_input,
        out.color,
    );

    return out;

#endif
}