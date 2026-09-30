#import bevy_pbr::forward_io::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var base_color_texture: texture_2d<f32>;

@group(#{MATERIAL_BIND_GROUP}) @binding(1)
var base_color_sampler: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(2)
var normal_texture: texture_2d<f32>;

@group(#{MATERIAL_BIND_GROUP}) @binding(3)
var normal_sampler: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(4)
var height_texture: texture_2d<f32>;

@group(#{MATERIAL_BIND_GROUP}) @binding(5)
var height_sampler: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(6)
var ambient_occlusion_texture: texture_2d<f32>;

@group(#{MATERIAL_BIND_GROUP}) @binding(7)
var ambient_occlusion_sampler: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(8)
var roughness_texture: texture_2d<f32>;

@group(#{MATERIAL_BIND_GROUP}) @binding(9)
var roughness_sampler: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(10)
var metallic_texture: texture_2d<f32>;

@group(#{MATERIAL_BIND_GROUP}) @binding(11)
var metallic_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let base_color = textureSample(
        base_color_texture,
        base_color_sampler,
        in.uv,
    );

    return base_color;
}