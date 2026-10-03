use bevy_egui::egui;

use crate::{
    data::{Atlas, TextureGroup},
    rendering::{Channel, ChannelSource, MaterialInput, RenderingUsage, TextureSource},
    ui::EditorUi,
};

pub fn rendering_panel(ui: &mut egui::Ui, editor_ui: &mut EditorUi, _atlas: &mut Atlas) {
    ui.heading("Rendering");

    let groups = &editor_ui.project_settings.texture_naming.groups;

    if groups.is_empty() {
        editor_ui.rendering_group = None;
        ui.label("No texture groups defined.");
        return;
    }

    // Select the first group if nothing is currently selected.
    let selection_is_valid = editor_ui
        .rendering_group
        .as_deref()
        .is_some_and(|id| groups.iter().any(|group| group.id == id));

    if !selection_is_valid {
        editor_ui.rendering_group = Some(groups[0].id.clone());
    }

    let Some(selected_id) = editor_ui.rendering_group.as_deref() else {
        return;
    };

    let Some(selected_index) = groups.iter().position(|group| group.id == selected_id) else {
        return;
    };

    let selected_name = groups[selected_index].name.clone();

    egui::ComboBox::from_id_salt("rendering_group")
        .selected_text(selected_name)
        .show_ui(ui, |ui| {
            for group in groups {
                ui.selectable_value(
                    &mut editor_ui.rendering_group,
                    Some(group.id.clone()),
                    &group.name,
                );
            }
        });

    ui.separator();

    let source_groups = groups
        .iter()
        .map(|group| (group.id.clone(), group.name.clone()))
        .collect::<Vec<_>>();

    let group = &mut editor_ui.project_settings.texture_naming.groups[selected_index];

    let before = group.rendering.clone();

    for input in MaterialInput::ALL {
        material_input_row(ui, group, input, &source_groups);
    }

    if group.rendering != before {
        editor_ui.rendering_revision = editor_ui.rendering_revision.wrapping_add(1);
    }
}

fn material_input_row(
    ui: &mut egui::Ui,
    group: &mut TextureGroup,
    input: MaterialInput,
    source_groups: &[(String, String)],
) {
    match input.usage() {
        RenderingUsage::Sample => {
            sample_input_row(ui, group, input, source_groups);
        }

        RenderingUsage::Channel => {
            channel_input_row(ui, group, input, source_groups);
        }
    }
}

fn sample_input_row(
    ui: &mut egui::Ui,
    group: &mut TextureGroup,
    input: MaterialInput,
    source_groups: &[(String, String)],
) {
    let mut selected_texture =
        texture_source_ref(group, input).map(|source| source.texture_id.clone());

    ui.horizontal(|ui| {
        ui.label(input.name());

        egui::ComboBox::from_id_salt(("rendering_texture", input))
            .selected_text(
                selected_texture
                    .as_deref()
                    .and_then(|id| {
                        source_groups
                            .iter()
                            .find(|(group_id, _)| group_id == id)
                            .map(|(_, name)| name.as_str())
                    })
                    .unwrap_or("None"),
            )
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(selected_texture.is_none(), "None")
                    .clicked()
                {
                    selected_texture = None;
                }

                for (group_id, group_name) in source_groups {
                    if ui
                        .selectable_label(
                            selected_texture.as_deref() == Some(group_id.as_str()),
                            group_name,
                        )
                        .clicked()
                    {
                        selected_texture = Some(group_id.clone());
                    }
                }
            });
    });

    *texture_source_mut(group, input) =
        selected_texture.map(|texture_id| TextureSource { texture_id });
}

fn channel_input_row(
    ui: &mut egui::Ui,
    group: &mut TextureGroup,
    input: MaterialInput,
    source_groups: &[(String, String)],
) {
    let (mut selected_texture, mut selected_channel) = match channel_source_ref(group, input) {
        Some(source) => (Some(source.texture_id.clone()), source.channel),
        None => (None, Channel::Red),
    };

    ui.horizontal(|ui| {
        ui.label(input.name());

        egui::ComboBox::from_id_salt(("rendering_texture", input))
            .selected_text(
                selected_texture
                    .as_deref()
                    .and_then(|id| {
                        source_groups
                            .iter()
                            .find(|(group_id, _)| group_id == id)
                            .map(|(_, name)| name.as_str())
                    })
                    .unwrap_or("None"),
            )
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(selected_texture.is_none(), "None")
                    .clicked()
                {
                    selected_texture = None;
                }

                for (group_id, group_name) in source_groups {
                    if ui
                        .selectable_label(
                            selected_texture.as_deref() == Some(group_id.as_str()),
                            group_name,
                        )
                        .clicked()
                    {
                        selected_texture = Some(group_id.clone());
                    }
                }
            });

        if selected_texture.is_some() {
            egui::ComboBox::from_id_salt(("rendering_channel", input))
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

    *channel_source_mut(group, input) = selected_texture.map(|texture_id| ChannelSource {
        texture_id,
        channel: selected_channel,
    });
}

fn texture_source_ref(group: &TextureGroup, input: MaterialInput) -> Option<&TextureSource> {
    match input {
        MaterialInput::BaseColor => group.rendering.base_color.as_ref(),
        MaterialInput::Normal => group.rendering.normal.as_ref(),

        MaterialInput::Height
        | MaterialInput::AmbientOcclusion
        | MaterialInput::Roughness
        | MaterialInput::Metallic => None,
    }
}

fn texture_source_mut(
    group: &mut TextureGroup,
    input: MaterialInput,
) -> &mut Option<TextureSource> {
    match input {
        MaterialInput::BaseColor => &mut group.rendering.base_color,
        MaterialInput::Normal => &mut group.rendering.normal,

        MaterialInput::Height
        | MaterialInput::AmbientOcclusion
        | MaterialInput::Roughness
        | MaterialInput::Metallic => {
            unreachable!("Channel input cannot use TextureSource")
        }
    }
}

fn channel_source_ref(group: &TextureGroup, input: MaterialInput) -> Option<&ChannelSource> {
    match input {
        MaterialInput::Height => group.rendering.height.as_ref(),
        MaterialInput::AmbientOcclusion => group.rendering.ambient_occlusion.as_ref(),
        MaterialInput::Roughness => group.rendering.roughness.as_ref(),
        MaterialInput::Metallic => group.rendering.metallic.as_ref(),

        MaterialInput::BaseColor | MaterialInput::Normal => None,
    }
}

fn channel_source_mut(
    group: &mut TextureGroup,
    input: MaterialInput,
) -> &mut Option<ChannelSource> {
    match input {
        MaterialInput::Height => &mut group.rendering.height,
        MaterialInput::AmbientOcclusion => &mut group.rendering.ambient_occlusion,
        MaterialInput::Roughness => &mut group.rendering.roughness,
        MaterialInput::Metallic => &mut group.rendering.metallic,

        MaterialInput::BaseColor | MaterialInput::Normal => {
            unreachable!("Sample input cannot use ChannelSource")
        }
    }
}
