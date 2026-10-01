#import bevy_pbr::forward_io::VertexOutput

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
    // This is temporary; these dimensions should eventually come
    // from the material/atlas rather than being hard-coded.
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

    let height = textureLoad(
        height_texture,
        pixel,
        0,
    ).r;

    let ambient_occlusion = textureLoad(
        ambient_occlusion_texture,
        pixel,
        0,
    ).r;

    let roughness = textureLoad(
        roughness_texture,
        pixel,
        0,
    ).r;

    let metallic = textureLoad(
        metallic_texture,
        pixel,
        0,
    ).r;

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

    let _roughness = roughness;
    let _height = height;

    return vec4<f32>(
        color,
        base_color.a
    );
}