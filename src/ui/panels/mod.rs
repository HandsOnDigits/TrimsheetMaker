mod toolbar;
use toolbar::toolbar;

mod materials;
use materials::materials_panel;

mod properties;

mod inspector;
use inspector::inspector_panel;

mod atlas;

mod rendering;

mod status_bar;
use status_bar::status_bar;

use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_egui::{EguiContexts, EguiTextureHandle, egui};

mod mesh;
use mesh::mesh_panel;

use super::EditorUi;

use crate::{
    data::Atlas,
    history::EditorHistory,
    hotkeys::{EditorActionMessage, HotkeyManager},
    mesh::{MeshLight, MeshSceneState},
    ui::windows::{self, floating_context_menus::AtlasContextMenu},
    viewport::{MeshPreview, MeshViewportContextMenu, MeshViewportInput, MeshViewportLayout},
};

#[derive(SystemParam)]
pub struct EditorUiContextMenus<'w> {
    viewport: ResMut<'w, MeshViewportContextMenu>,
    atlas: ResMut<'w, AtlasContextMenu>,
}

pub fn editor_ui(
    mut contexts: EguiContexts,
    mut editor_ui: ResMut<EditorUi>,
    mut atlas: ResMut<Atlas>,
    mut images: ResMut<Assets<Image>>,
    mut hotkeys: ResMut<HotkeyManager>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mesh_viewport: ResMut<MeshViewportLayout>,
    mut mesh_preview: ResMut<MeshPreview>,
    mut mesh_light: ResMut<MeshLight>,
    mut mesh_input: ResMut<MeshViewportInput>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mesh_scene_state: ResMut<MeshSceneState>,
    mut action_messages: MessageWriter<EditorActionMessage>,
    mut history: ResMut<EditorHistory>,
    mut context_menus: EditorUiContextMenus,
) -> Result {
    mesh_input.pointer_over_ui = false;
    mesh_input.floating_window_dragging = false;

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

    #[cfg(debug_assertions)]
    ctx.global_style_mut(|style| {
        style.debug.show_unaligned = false;
    });

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
            toolbar(
                ui,
                &mut editor_ui,
                &mut atlas,
                &mut images,
                &hotkeys,
                &mut mesh_preview,
                &mut mesh_light,
                &mut mesh_scene_state,
                &mut commands,
                &mut meshes,
                &mut *history,
            );
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
                materials_panel(
                    ui,
                    &mut editor_ui,
                    &mut atlas,
                    &mut *history,
                    &mesh_scene_state,
                );
            });
    }

    if editor_ui.show_inspector {
        if editor_ui.show_inspector {
            egui::Panel::right("inspector_panel")
                .resizable(true)
                .default_size(320.0)
                .size_range(220.0..=500.0)
                .show(&mut root_ui, |ui| {
                    inspector_panel(
                        ui,
                        &mut editor_ui,
                        &mut atlas,
                        &mut *history,
                        &mut mesh_scene_state,
                    );
                });
        }
    }

    if editor_ui.show_editor_settings_window {
        mesh_input.floating_window_dragging |= windows::editor_settings::editor_settings_window(
            ctx,
            &mut editor_ui,
            &mut hotkeys,
            &keyboard,
        );
    }

    let available = root_ui.available_rect_before_wrap();

    let divider_width = 5.0;

    let split_x = available.left() + available.width() * editor_ui.mesh_view_split;

    // Left: 3D mesh viewport.
    let mesh_rect = egui::Rect::from_min_max(
        available.min,
        egui::pos2(split_x - divider_width * 0.5, available.bottom()),
    );

    // Vertical divider.
    let divider_rect = egui::Rect::from_min_max(
        egui::pos2(split_x - divider_width * 0.5, available.top()),
        egui::pos2(split_x + divider_width * 0.5, available.bottom()),
    );

    // Right: 2D atlas.
    let atlas_rect = egui::Rect::from_min_max(
        egui::pos2(split_x + divider_width * 0.5, available.top()),
        available.max,
    );

    root_ui.scope_builder(egui::UiBuilder::new().max_rect(mesh_rect), |ui| {
        mesh_panel(
            ui,
            &mut mesh_preview,
            &mut mesh_light,
            &mut mesh_input,
            &hotkeys,
            &mut mesh_viewport,
        );
    });

    // Draggable divider.
    let divider_response = root_ui.interact(
        divider_rect,
        root_ui.make_persistent_id("mesh_atlas_split"),
        egui::Sense::drag(),
    );

    if divider_response.dragged() {
        editor_ui.mesh_view_split = (editor_ui.mesh_view_split
            + divider_response.drag_delta().x / available.width())
        .clamp(0.2, 0.8);
    }

    if divider_response.hovered() || divider_response.dragged() {
        root_ui
            .ctx()
            .set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
    }

    root_ui.painter().rect_filled(
        divider_rect,
        0.0,
        root_ui.visuals().widgets.noninteractive.bg_fill,
    );

    root_ui.scope_builder(egui::UiBuilder::new().max_rect(atlas_rect), |ui| {
        atlas::atlas_view::atlas_view(
            ui,
            &mut editor_ui,
            &mut atlas,
            &keyboard,
            &mut mesh_scene_state,
            &mut *history,
            &mut mesh_preview,
            &mut context_menus.atlas,
        );
    });

    // Bulk/folder import.
    mesh_input.floating_window_dragging |=
        windows::import_multiple_materials::import_multiple_materials_window(
            ctx,
            &mut editor_ui,
            &mut atlas,
            &mut images,
        );

    // Single-material import.
    mesh_input.floating_window_dragging |=
        windows::import_single_material::import_single_material_window(
            ctx,
            &mut editor_ui,
            &mut atlas,
            &mut images,
        );

    // Mesh import.
    mesh_input.floating_window_dragging |= windows::import_mesh::import_mesh_window(
        ctx,
        &mut editor_ui,
        &mut commands,
        &mut meshes,
        &mut mesh_scene_state,
        &mut mesh_preview,
    );

    // Mesh list.
    mesh_input.floating_window_dragging |= windows::meshes::meshes_window(
        ctx,
        &mut editor_ui,
        &mut atlas,
        &mut mesh_scene_state,
        &mut mesh_preview,
        &mut action_messages,
        &mut *history,
    );

    windows::atlas_context_menu(ctx, &mut context_menus.atlas, &mut action_messages);

    // 3D viewport primitive context menu.
    windows::viewport_primitive_context_menu(
        ctx,
        &mut context_menus.viewport,
        &mut action_messages,
    );

    // Prevent viewport interactions while an egui popup or dropdown is open.
    mesh_input.pointer_over_ui |= egui::Popup::is_any_open(ctx);

    Ok(())
}
