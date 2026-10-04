use bevy_egui::egui;

use crate::{
    data::{Atlas, TextureGroup},
    rendering::{Channel, ChannelSource, MaterialInput, RenderingUsage},
    ui::EditorUi,
};

pub fn rendering_panel(ui: &mut egui::Ui, editor_ui: &mut EditorUi, _atlas: &mut Atlas) {
    ui.heading("Rendering");

    let groups = &mut editor_ui.project_settings.texture_naming.groups;

    if groups.is_empty() {
        ui.label("No texture groups.");
        return;
    }

    for group in groups {
        ui.separator();
        ui.heading(&group.name);

        let before = group.rendering.clone();

        for input in MaterialInput::ALL {
            material_input_row(ui, group, input);
        }

        if group.rendering != before {
            editor_ui.rendering_revision = editor_ui.rendering_revision.wrapping_add(1);
        }
    }
}

fn material_input_row(ui: &mut egui::Ui, group: &mut TextureGroup, input: MaterialInput) {
    match input.usage() {
        RenderingUsage::Sample => {
            sample_input_row(ui, group, input);
        }
        RenderingUsage::Channel => {
            channel_input_row(ui, group, input);
        }
    }
}

fn sample_input_row(ui: &mut egui::Ui, group: &mut TextureGroup, input: MaterialInput) {
    let enabled = match input {
        MaterialInput::BaseColor => &mut group.rendering.base_color,
        MaterialInput::Normal => &mut group.rendering.normal,
        _ => unreachable!("channel input cannot use sample binding"),
    };

    ui.checkbox(enabled, input.name());
}

fn channel_input_row(ui: &mut egui::Ui, group: &mut TextureGroup, input: MaterialInput) {
    let binding = match input {
        MaterialInput::AmbientOcclusion => &mut group.rendering.ambient_occlusion,
        MaterialInput::Roughness => &mut group.rendering.roughness,
        MaterialInput::Metallic => &mut group.rendering.metallic,
        _ => unreachable!("sample input cannot use channel binding"),
    };

    let mut enabled = binding.is_some();

    let mut channel = binding
        .as_ref()
        .map(|source| source.channel)
        .unwrap_or(Channel::Red);

    ui.horizontal(|ui| {
        ui.checkbox(&mut enabled, input.name());

        if enabled {
            egui::ComboBox::from_id_salt(("rendering_channel", &group.id, input))
                .selected_text(channel.name())
                .show_ui(ui, |ui| {
                    for candidate in Channel::ALL {
                        ui.selectable_value(&mut channel, candidate, candidate.name());
                    }
                });
        }
    });

    *binding = enabled.then_some(ChannelSource { channel });
}
