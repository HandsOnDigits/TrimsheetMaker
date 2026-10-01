#import bevy_pbr::forward_io::VertexOutput

struct MeshPreviewGraph {
    base_color: vec4<f32>,
    normal: vec4<f32>,
    height: vec4<f32>,
    ambient_occlusion: vec4<f32>,
    roughness: vec4<f32>,
    metallic: vec4<f32>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var base_color_texture: texture_2d<f32>;

@group(#{MATERIAL_BIND_GROUP}) @binding(1)
var base_color_sampler: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(2)
var normal_texture: texture_2d<f32>;

@group(#{MATERIAL_BIND_GROUP}) @binding(3)
var height_texture: texture_2d<f32>;

@group(#{MATERIAL_BIND_GROUP}) @binding(4)
var ambient_occlusion_texture: texture_2d<f32>;

@group(#{MATERIAL_BIND_GROUP}) @binding(5)
var roughness_texture: texture_2d<f32>;

@group(#{MATERIAL_BIND_GROUP}) @binding(6)
var metallic_texture: texture_2d<f32>;

@group(#{MATERIAL_BIND_GROUP}) @binding(7)
var<uniform> graph: MeshPreviewGraph;


fn read_channel(
    value: vec4<f32>,
    channel: u32,
) -> f32 {
    switch channel {
        case 0u: {
            return value.r;
        }

        case 1u: {
            return value.g;
        }

        case 2u: {
            return value.b;
        }

        default: {
            return value.a;
        }
    }
}


@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;

    let base_color = textureSample(
        base_color_texture,
        base_color_sampler,
        uv,
    );

    // Convert normalized UV coordinates to atlas texel coordinates.
    //
    // Temporary hard-coded dimensions. These should eventually be
    // supplied as part of the material/atlas data.
    let atlas_size = vec2<f32>(544.0, 3616.0);

    let pixel = vec2<i32>(
        clamp(
            uv * atlas_size,
            vec2<f32>(0.0),
            atlas_size - vec2<f32>(1.0),
        )
    );

    let normal_sample = textureLoad(
        normal_texture,
        pixel,
        0,
    );

    let height_sample = textureLoad(
        height_texture,
        pixel,
        0,
    );

    let ambient_occlusion_sample = textureLoad(
        ambient_occlusion_texture,
        pixel,
        0,
    );

    let roughness_sample = textureLoad(
        roughness_texture,
        pixel,
        0,
    );

    let metallic_sample = textureLoad(
        metallic_texture,
        pixel,
        0,
    );

    // The graph determines which channel supplies each scalar value.
    let height = read_channel(
        height_sample,
        u32(graph.height.x),
    );
    
    let ambient_occlusion = read_channel(
        ambient_occlusion_sample,
        u32(graph.ambient_occlusion.x),
    );
    
    let roughness = read_channel(
        roughness_sample,
        u32(graph.roughness.x),
    );
    
    let metallic = read_channel(
        metallic_sample,
        u32(graph.metallic.x),
    );

    // Convert the normal texture from [0, 1] to [-1, 1].
    let normal = normalize(
        normal_sample.xyz * 2.0 - vec3<f32>(1.0)
    );

    let light_direction = normalize(
        vec3<f32>(0.4, 0.7, 0.6)
    );

    let diffuse = max(
        dot(normal, light_direction),
        0.0
    );

    let ambient = 0.15;

    let lighting =
        ambient * ambient_occlusion +
        diffuse;

    let diffuse_factor = 1.0 - metallic;

    let color =
        base_color.rgb *
        (lighting * diffuse_factor + metallic);

    // These are currently evaluated so the graph is functional,
    // but height/roughness are not yet affecting the lighting model.
    let _height = height;
    let _roughness = roughness;

    return vec4<f32>(
        color,
        base_color.a
    );
}