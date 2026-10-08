use bevy::{
    material::OpaqueRendererMethod,
    pbr::{ExtendedMaterial, MaterialExtension, StandardMaterial},
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
};

use crate::{
    data::{Atlas, AtlasMaterial, TextureNaming},
    rendering::DefaultTextures,
    ui::EditorUi,
};

use super::state::MeshPreviewMaterials;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub struct MeshPreviewExtension {
    #[texture(100)]
    #[sampler(101)]
    pub base_color_texture: Option<Handle<Image>>,

    #[texture(102)]
    #[sampler(103)]
    pub normal_texture: Option<Handle<Image>>,

    #[texture(104)]
    #[sampler(105)]
    pub ambient_occlusion_texture: Option<Handle<Image>>,

    #[texture(106)]
    #[sampler(107)]
    pub roughness_texture: Option<Handle<Image>>,

    #[texture(108)]
    #[sampler(109)]
    pub metallic_texture: Option<Handle<Image>>,
}

impl MaterialExtension for MeshPreviewExtension {
    fn fragment_shader() -> ShaderRef {
        "shaders/mesh_preview_material.wgsl".into()
    }

    fn enable_shadows() -> bool {
        true
    }
}

pub type MeshPreviewMaterial = ExtendedMaterial<StandardMaterial, MeshPreviewExtension>;

pub fn sync_atlas_to_mesh_preview_material(
    atlas: Res<Atlas>,
    editor_ui: Res<EditorUi>,
    default_textures: Res<DefaultTextures>,
    mut materials: ResMut<Assets<MeshPreviewMaterial>>,
    mut preview_materials: ResMut<MeshPreviewMaterials>,
) {
    for atlas_material in &atlas.materials {
        let handle = if let Some(handle) = preview_materials
            .by_atlas_material
            .get(&atlas_material.index)
        {
            handle.clone()
        } else {
            let handle = materials.add(MeshPreviewMaterial {
                base: StandardMaterial {
                    opaque_render_method: OpaqueRendererMethod::Forward,
                    ..default()
                },
                extension: MeshPreviewExtension::default(),
            });

            preview_materials
                .by_atlas_material
                .insert(atlas_material.index, handle.clone());

            handle
        };

        // Normal material.
        if let Some(mut material) = materials.get_mut(&handle) {
            sync_preview_material(
                &mut material.extension,
                atlas_material,
                &atlas,
                &editor_ui.project_settings.texture_naming,
                &default_textures,
            );
        }
    }
}

fn sync_preview_material(
    extension: &mut MeshPreviewExtension,
    atlas_material: &AtlasMaterial,
    atlas: &Atlas,
    texture_naming: &TextureNaming,
    default_textures: &DefaultTextures,
) {
    let mut has_base_color = false;
    let mut has_normal = false;
    let mut has_ao = false;
    let mut has_roughness = false;
    let mut has_metallic = false;

    for texture in &atlas_material.textures {
        let Some(group_id) = texture.group_id.as_deref() else {
            continue;
        };

        let Some(group) = texture_naming
            .groups
            .iter()
            .find(|group| group.id == group_id)
        else {
            continue;
        };

        let rendering = &group.rendering;

        has_base_color |= rendering.base_color;
        has_normal |= rendering.normal;
        has_ao |= rendering.ambient_occlusion.is_some();
        has_roughness |= rendering.roughness.is_some();
        has_metallic |= rendering.metallic.is_some();
    }

    extension.base_color_texture = if has_base_color {
        atlas.images.base_color.clone()
    } else {
        Some(default_textures.checkerboard.clone())
    };

    extension.normal_texture = if has_normal {
        atlas.images.normal.clone()
    } else {
        Some(default_textures.flat_normal.clone())
    };

    extension.ambient_occlusion_texture = if has_ao {
        atlas.images.ambient_occlusion.clone()
    } else {
        Some(default_textures.white.clone())
    };

    extension.roughness_texture = if has_roughness {
        atlas.images.roughness.clone()
    } else {
        Some(default_textures.white.clone())
    };

    extension.metallic_texture = if has_metallic {
        atlas.images.metallic.clone()
    } else {
        Some(default_textures.black.clone())
    };
}
