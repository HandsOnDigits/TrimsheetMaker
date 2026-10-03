use bevy_egui::egui;

use crate::{
    data::{Atlas, AtlasMaterial},
    rendering::{Channel, ChannelSource, MaterialChannel},
    ui::EditorUi,
};

pub fn rendering_panel(ui: &mut egui::Ui, editor_ui: &mut EditorUi, atlas: &mut Atlas) {
    ui.heading("Rendering");

    // Material group selector.
    ui.horizontal(|ui| {
        ui.label("Material Group");

        let selected_name = editor_ui
            .rendering_material
            .and_then(|index| editor_ui.project_settings.texture_naming.groups.get(index))
            .map(|group| group.name.as_str())
            .unwrap_or("None");

        egui::ComboBox::from_id_salt("rendering_material_group")
            .selected_text(selected_name)
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(editor_ui.rendering_material.is_none(), "None")
                    .clicked()
                {
                    editor_ui.rendering_material = None;
                }

                for (index, group) in editor_ui
                    .project_settings
                    .texture_naming
                    .groups
                    .iter()
                    .enumerate()
                {
                    if ui
                        .selectable_label(editor_ui.rendering_material == Some(index), &group.name)
                        .clicked()
                    {
                        editor_ui.rendering_material = Some(index);
                    }
                }
            });
    });

    ui.separator();

    let Some(group_index) = editor_ui.rendering_material else {
        ui.label("Select a material group.");
        return;
    };

    // Find an atlas material belonging to this material group.
    //
    // This is only appropriate if AtlasMaterial is still where the
    // RenderingMaterialBindings are stored.
    let Some(group) = editor_ui
        .project_settings
        .texture_naming
        .groups
        .get(group_index)
    else {
        editor_ui.rendering_material = None;
        return;
    };

    let Some(material_index) = atlas.materials.iter().position(|material| {
        material
            .textures
            .iter()
            .any(|texture| texture.id == group.id)
    }) else {
        ui.label("No material uses this material group.");
        return;
    };

    let texture_ids = atlas.materials[material_index]
        .textures
        .iter()
        .map(|texture| texture.id.clone())
        .collect::<Vec<_>>();

    let material = &mut atlas.materials[material_index];

    for output in MaterialChannel::ALL {
        material_channel_row(ui, material, output, &texture_ids);
    }
}

fn material_channel_row(
    ui: &mut egui::Ui,
    material: &mut AtlasMaterial,
    output: MaterialChannel,
    texture_ids: &[String],
) {
    let (current_texture, current_channel) = {
        let binding = binding_ref(material, output);

        (
            binding.map(|source| source.texture_id.clone()),
            binding.map(|source| source.channel).unwrap_or(Channel::Red),
        )
    };

    let mut selected_texture = current_texture;
    let mut selected_channel = current_channel;

    ui.horizontal(|ui| {
        ui.label(output.name());

        egui::ComboBox::from_id_salt(("rendering_texture", output))
            .selected_text(selected_texture.as_deref().unwrap_or("None"))
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(selected_texture.is_none(), "None")
                    .clicked()
                {
                    selected_texture = None;
                }

                for texture_id in texture_ids {
                    if ui
                        .selectable_label(
                            selected_texture.as_deref() == Some(texture_id.as_str()),
                            texture_id,
                        )
                        .clicked()
                    {
                        selected_texture = Some(texture_id.clone());
                    }
                }
            });

        if selected_texture.is_some() {
            egui::ComboBox::from_id_salt(("rendering_channel", output))
                .selected_text(selected_channel.name())
                .show_ui(ui, |ui| {
                    for channel in Channel::ALL {
                        if ui
                            .selectable_label(selected_channel == channel, channel.name())
                            .clicked()
                        {
                            selected_channel = channel;
                        }
                    }
                });
        }
    });

    *binding_mut(material, output) = selected_texture.map(|texture_id| ChannelSource {
        texture_id,
        channel: selected_channel,
    });
}

fn binding_ref(material: &AtlasMaterial, channel: MaterialChannel) -> Option<&ChannelSource> {
    match channel {
        MaterialChannel::BaseColor => material.rendering.base_color.as_ref(),
        MaterialChannel::Normal => material.rendering.normal.as_ref(),
        MaterialChannel::Height => material.rendering.height.as_ref(),
        MaterialChannel::AmbientOcclusion => material.rendering.ambient_occlusion.as_ref(),
        MaterialChannel::Roughness => material.rendering.roughness.as_ref(),
        MaterialChannel::Metallic => material.rendering.metallic.as_ref(),
    }
}

fn binding_mut(
    material: &mut AtlasMaterial,
    channel: MaterialChannel,
) -> &mut Option<ChannelSource> {
    match channel {
        MaterialChannel::BaseColor => &mut material.rendering.base_color,
        MaterialChannel::Normal => &mut material.rendering.normal,
        MaterialChannel::Height => &mut material.rendering.height,
        MaterialChannel::AmbientOcclusion => &mut material.rendering.ambient_occlusion,
        MaterialChannel::Roughness => &mut material.rendering.roughness,
        MaterialChannel::Metallic => &mut material.rendering.metallic,
    }
}
