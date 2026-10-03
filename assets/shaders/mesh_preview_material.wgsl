#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions,
    pbr_types,
    pbr_functions::{
        apply_pbr_lighting,
        main_pass_post_lighting_processing,
    },
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{
        VertexOutput,
        FragmentOutput,
    },
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{
        VertexOutput,
        FragmentOutput,
    },
}
#endif


// -----------------------------------------------------------------------------
// Mesh Preview Extension
// -----------------------------------------------------------------------------

@group(#{MATERIAL_BIND_GROUP}) @binding(100)
var base_color_texture: texture_2d<f32>;

@group(#{MATERIAL_BIND_GROUP}) @binding(101)
var base_color_sampler: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(102)
var normal_texture: texture_2d<f32>;

@group(#{MATERIAL_BIND_GROUP}) @binding(103)
var normal_sampler: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(104)
var ambient_occlusion_texture: texture_2d<f32>;

@group(#{MATERIAL_BIND_GROUP}) @binding(105)
var ambient_occlusion_sampler: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(106)
var roughness_texture: texture_2d<f32>;

@group(#{MATERIAL_BIND_GROUP}) @binding(107)
var roughness_sampler: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(108)
var metallic_texture: texture_2d<f32>;

@group(#{MATERIAL_BIND_GROUP}) @binding(109)
var metallic_sampler: sampler;


// -----------------------------------------------------------------------------
// Fragment
// -----------------------------------------------------------------------------

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {

    var pbr_input =
        pbr_input_from_standard_material(
            in,
            is_front,
        );

    #ifdef VERTEX_UVS

    // -------------------------------------------------------------------------
    // Base Color
    // -------------------------------------------------------------------------

    pbr_input.material.base_color =
        textureSample(
            base_color_texture,
            base_color_sampler,
            in.uv,
        );


    // -------------------------------------------------------------------------
    // Normal
    //
    // Geometry displacement has already happened in the compute stage.
    // The normal map is applied here using the displaced mesh tangent basis.
    // -------------------------------------------------------------------------

    #ifndef LOAD_PREPASS_NORMALS
    #ifdef VERTEX_TANGENTS

    let normal_sample =
        textureSample(
            normal_texture,
            normal_sampler,
            in.uv,
        ).rgb;

    let tbn =
        pbr_functions::calculate_tbn_mikktspace(
            pbr_input.world_normal,
            in.world_tangent,
        );

    pbr_input.N =
        pbr_functions::apply_normal_mapping(
            pbr_input.flags,
            tbn,
            false,
            is_front,
            normal_sample,
        );

    pbr_input.clearcoat_N =
        pbr_input.N;

    #endif
    #endif


    // -------------------------------------------------------------------------
    // Ambient Occlusion
    // -------------------------------------------------------------------------

    let ambient_occlusion =
        textureSample(
            ambient_occlusion_texture,
            ambient_occlusion_sampler,
            in.uv,
        ).r;

    pbr_input.diffuse_occlusion =
        vec3<f32>(ambient_occlusion);


    // -------------------------------------------------------------------------
    // Roughness
    // -------------------------------------------------------------------------

    let roughness =
        textureSample(
            roughness_texture,
            roughness_sampler,
            in.uv,
        ).r;

    pbr_input.material.perceptual_roughness =
        roughness;


    // -------------------------------------------------------------------------
    // Metallic
    // -------------------------------------------------------------------------

    let metallic =
        textureSample(
            metallic_texture,
            metallic_sampler,
            in.uv,
        ).r;

    pbr_input.material.metallic =
        metallic;

    #endif


    // -------------------------------------------------------------------------
    // Forward / Deferred
    // -------------------------------------------------------------------------

    #ifdef PREPASS_PIPELINE

    return deferred_output(
        in,
        pbr_input,
    );

    #else

    var out: FragmentOutput;

    out.color =
        apply_pbr_lighting(
            pbr_input,
        );

    out.color =
        main_pass_post_lighting_processing(
            pbr_input,
            out.color,
        );

    return out;

    #endif
}