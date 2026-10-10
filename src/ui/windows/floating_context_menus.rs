use bevy::prelude::*;
use bevy_egui::egui;

use crate::{
    data::Atlas,
    history::EditorHistory,
    hotkeys::{EditorAction, EditorActionMessage},
    linking::assign_selected_primitives,
    mesh::MeshSceneState,
    ui::EditorUi,
    viewport::{MeshPreview, MeshViewportContextMenu},
};

#[derive(Resource, Default)]
pub struct AtlasContextMenu {
    pub region_index: Option<usize>,
    pub position: Vec2,
    pub open: bool,
}

pub fn viewport_primitive_context_menu(
    ctx: &egui::Context,
    menu: &mut MeshViewportContextMenu,
    action_messages: &mut MessageWriter<EditorActionMessage>,
) {
    if !menu.open {
        return;
    }

    let Some(selection) = menu.selection else {
        menu.open = false;
        return;
    };

    let response = egui::Area::new(egui::Id::new("viewport_primitive_context_menu"))
        .order(egui::Order::Foreground)
        .fixed_pos(egui::pos2(menu.position.x, menu.position.y))
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style())
                .show(ui, |ui| {
                    ui.set_min_width(190.0);

                    if ui.button("Delete Primitive").clicked() {
                        action_messages.write(EditorActionMessage::new(
                            EditorAction::RemoveMeshPrimitiveAt {
                                mesh_id: selection.mesh_id,
                                object_index: selection.object_index,
                                primitive_index: selection.primitive_index,
                            },
                        ));
                        menu.open = false;
                    }

                    if ui.button("Remove Material Assignment").clicked() {
                        action_messages.write(EditorActionMessage::new(
                            EditorAction::UnassignMeshPrimitive {
                                mesh_id: selection.mesh_id,
                                object_index: selection.object_index,
                                primitive_index: selection.primitive_index,
                            },
                        ));
                        menu.open = false;
                    }
                })
                .response
        });

    let menu_rect = response.inner.rect;

    ctx.input(|input| {
        if input.key_pressed(egui::Key::Escape) {
            menu.open = false;
        }

        if input.pointer.any_click()
            && let Some(pointer_pos) = input.pointer.interact_pos()
            && !menu_rect.contains(pointer_pos)
        {
            menu.open = false;
        }
    });

    if !menu.open {
        menu.selection = None;
    }
}

pub fn atlas_context_menu(
    ctx: &egui::Context,
    menu: &mut AtlasContextMenu,
    action_messages: &mut MessageWriter<EditorActionMessage>,
    editor_ui: &mut EditorUi,
    atlas: &mut Atlas,
    mesh_scene_state: &MeshSceneState,
    mesh_preview: &MeshPreview,
    history: &mut EditorHistory,
) {
    if !menu.open {
        return;
    }

    let Some(region_index) = menu.region_index else {
        menu.open = false;
        return;
    };

    let response = egui::Area::new(egui::Id::new("atlas_context_menu"))
        .order(egui::Order::Foreground)
        .fixed_pos(egui::pos2(menu.position.x, menu.position.y))
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style())
                .show(ui, |ui| {
                    ui.set_min_width(190.0);

                    if ui.button("Assign to Selected Primitive").clicked() {
                        // Make the right-clicked region the inspected region,
                        // since the assignment helper uses that selection.
                        editor_ui.inspected_atlas_region = Some(region_index);

                        assign_selected_primitives(
                            editor_ui,
                            atlas,
                            mesh_scene_state,
                            mesh_preview,
                            history,
                        );

                        menu.open = false;
                    }

                    if ui.button("Delete Atlas Region").clicked() {
                        action_messages.write(EditorActionMessage::new(
                            EditorAction::RemoveAtlasRegionAt {
                                index: region_index,
                            },
                        ));

                        menu.open = false;
                    }
                })
                .response
        });

    let menu_rect = response.inner.rect;

    ctx.input(|input| {
        if input.key_pressed(egui::Key::Escape) {
            menu.open = false;
        }

        if input.pointer.any_click()
            && let Some(pointer_pos) = input.pointer.interact_pos()
            && !menu_rect.contains(pointer_pos)
        {
            menu.open = false;
        }
    });

    if !menu.open {
        menu.region_index = None;
    }
}
