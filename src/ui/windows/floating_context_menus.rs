use bevy::prelude::*;
use bevy_egui::egui;

use crate::{
    hotkeys::{EditorAction, EditorActionMessage},
    viewport::MeshViewportContextMenu,
};

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
                            EditorAction::RemoveMeshPrimitive {
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

pub fn floating_context_menus(
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

    let response = egui::Area::new(egui::Id::new("viewport_context_menu"))
        .order(egui::Order::Foreground)
        .fixed_pos(egui::pos2(menu.position.x, menu.position.y))
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style())
                .show(ui, |ui| {
                    ui.set_min_width(180.0);

                    if ui.button("Delete Primitive").clicked() {
                        action_messages.write(EditorActionMessage::new(
                            EditorAction::RemoveMeshPrimitive {
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
