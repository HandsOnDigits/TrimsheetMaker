mod toolbar;
use toolbar::toolbar;

mod materials;
use materials::materials_panel;

mod inspector;
use inspector::inspector_panel;

mod atlas;

mod status_bar;
use status_bar::status_bar;

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiTextureHandle, egui};

use super::EditorUi;

use crate::{
    data::Atlas,
    hotkeys::{EditorAction, HotkeyManager},
    mesh::MeshViewportLayout,
    ui::windows,
};

pub fn editor_ui(
    mut contexts: EguiContexts,
    mut editor_ui: ResMut<EditorUi>,
    mut atlas: ResMut<Atlas>,
    mut images: ResMut<Assets<Image>>,
    mut hotkeys: ResMut<HotkeyManager>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mesh_viewport: ResMut<MeshViewportLayout>,
) -> Result {
    for material in &mut atlas.materials {
        for texture in &mut material.textures {
            if texture.egui_texture.is_none() {
                if let Some(image) = &texture.image {
                    let texture_id = contexts.add_image(EguiTextureHandle::Weak(image.id()));

                    texture.egui_texture = Some(texture_id);
                }
            }
        }
    }

    let ctx = contexts.ctx_mut()?;

    let mut root_ui = egui::Ui::new(
        ctx.clone(),
        "trimsheet_maker_root".into(),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );

    egui::Panel::top("toolbar")
        .exact_size(44.0)
        .show(&mut root_ui, |ui| {
            toolbar(ui, &mut editor_ui, &mut atlas, &mut images, &hotkeys);
        });

    if editor_ui.show_status {
        egui::Panel::bottom("status_bar")
            .exact_size(26.0)
            .show(&mut root_ui, |ui| {
                status_bar(ui, &editor_ui, &atlas);
            });
    }

    if editor_ui.show_materials {
        egui::Panel::left("materials_panel")
            .resizable(true)
            .default_size(230.0)
            .size_range(180.0..=400.0)
            .show(&mut root_ui, |ui| {
                materials_panel(ui, &mut editor_ui, &mut atlas);
            });
    }

    if editor_ui.show_inspector {
        egui::Panel::right("inspector_panel")
            .resizable(true)
            .default_size(280.0)
            .size_range(220.0..=500.0)
            .show(&mut root_ui, |ui| {
                inspector_panel(ui, &mut editor_ui, &mut atlas);
            });
    }

    if editor_ui.show_editor_settings_window {
        windows::editor_settings::editor_settings_window(
            ctx,
            &mut editor_ui,
            &mut hotkeys,
            &keyboard,
        );
    }

    let available = root_ui.available_rect_before_wrap();

    let divider_height = 5.0;

    let split_y = available.top() + available.height() * editor_ui.mesh_view_split;

    let mesh_rect = egui::Rect::from_min_max(
        available.min,
        egui::pos2(available.right(), split_y - divider_height * 0.5),
    );

    let divider_rect = egui::Rect::from_min_max(
        egui::pos2(available.left(), split_y - divider_height * 0.5),
        egui::pos2(available.right(), split_y + divider_height * 0.5),
    );

    let atlas_rect = egui::Rect::from_min_max(
        egui::pos2(available.left(), split_y + divider_height * 0.5),
        available.max,
    );

    // Synchronize the native Bevy 3D viewport with the upper area.
    mesh_viewport.x = mesh_rect.left();
    mesh_viewport.y = mesh_rect.top();
    mesh_viewport.width = mesh_rect.width();
    mesh_viewport.height = mesh_rect.height();

    // Draggable divider.
    let divider_response = root_ui.interact(
        divider_rect,
        root_ui.make_persistent_id("mesh_atlas_split"),
        egui::Sense::drag(),
    );

    if divider_response.dragged() {
        editor_ui.mesh_view_split = (editor_ui.mesh_view_split
            + divider_response.drag_delta().y / available.height())
        .clamp(0.2, 0.8);
    }

    if divider_response.hovered() || divider_response.dragged() {
        root_ui
            .ctx()
            .set_cursor_icon(egui::CursorIcon::ResizeVertical);
    }

    root_ui.painter().rect_filled(
        divider_rect,
        0.0,
        root_ui.visuals().widgets.noninteractive.bg_fill,
    );

    // The existing atlas occupies the lower area.
    root_ui.scope_builder(egui::UiBuilder::new().max_rect(atlas_rect), |ui| {
        atlas::atlas_view::atlas_view(ui, &mut editor_ui, &mut atlas, &keyboard);
    });

    // Bulk/folder import.
    windows::import_multiple_materials::import_multiple_materials_window(
        ctx,
        &mut editor_ui,
        &mut atlas,
        &mut images,
    );

    // Single-material import.
    windows::import_single_material::import_single_material_window(
        ctx,
        &mut editor_ui,
        &mut atlas,
        &mut images,
    );

    Ok(())
}

fn menu_action_button(
    ui: &mut egui::Ui,
    hotkeys: &HotkeyManager,
    action: EditorAction,
    label: &str,
) -> bool {
    let mut button = egui::Button::new(label);

    if let Some(shortcut) = hotkeys.display_name(action) {
        button = button.shortcut_text(shortcut);
    }

    ui.add(button).clicked()
}
