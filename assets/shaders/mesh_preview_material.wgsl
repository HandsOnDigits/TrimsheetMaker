#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
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


// -----------------------------------------------------------------------------
// Height
// -----------------------------------------------------------------------------

struct MeshPreviewHeight {
    scale: f32,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(112)
var<uniform> height: MeshPreviewHeight;


// -----------------------------------------------------------------------------
// Fragment
// -----------------------------------------------------------------------------

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {

    // Let Bevy construct the complete PBR input.
    var pbr_input =
        pbr_input_from_standard_material(in, is_front);


    // -------------------------------------------------------------------------
    // Custom atlas material
    //
    // Start without parallax. This is deliberately the simplest possible
    // modification to Bevy's PBR input.
    // -------------------------------------------------------------------------

    #ifdef VERTEX_UVS

    pbr_input.material.base_color =
        textureSample(
            base_color_texture,
            base_color_sampler,
            in.uv,
        );


    let normal =
        textureSample(
            normal_texture,
            normal_sampler,
            in.uv,
        );

    // Bevy's StandardMaterial normal-map path expects the sampled value
    // in [0, 1]. Do NOT convert it to [-1, 1] here.
    //
    // We are intentionally leaving pbr_input.N alone for this first
    // working version. This proves that the custom atlas textures can
    // coexist with Bevy's normal/light/shadow pipeline.


    let ambient_occlusion =
        textureSample(
            ambient_occlusion_texture,
            ambient_occlusion_sampler,
            in.uv,
        ).r;

    let roughness =
        textureSample(
            roughness_texture,
            roughness_sampler,
            in.uv,
        ).r;

    let metallic =
        textureSample(
            metallic_texture,
            metallic_sampler,
            in.uv,
        ).r;


    pbr_input.diffuse_occlusion =
        vec3<f32>(ambient_occlusion);

    pbr_input.material.perceptual_roughness =
        roughness;

    pbr_input.material.metallic =
        metallic;

    #endif


    // -------------------------------------------------------------------------
    // Forward / deferred
    // -------------------------------------------------------------------------

    #ifdef PREPASS_PIPELINE

    // In deferred mode lighting happens later in the deferred lighting pass.
    return deferred_output(
        in,
        pbr_input,
    );

    #else

    var out: FragmentOutput;

    // Bevy performs:
    //
    //   directional lights
    //   point lights
    //   spot lights
    //   PBR BRDF
    //   shadows
    //   ambient lighting
    //
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