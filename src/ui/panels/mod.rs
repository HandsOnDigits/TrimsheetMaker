mod toolbar;
use toolbar::toolbar;

mod materials;
use materials::materials_panel;

mod inspector;
use inspector::inspector_panel;

mod atlas_view;
use atlas_view::atlas_view;

mod status_bar;
use status_bar::status_bar;

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiTextureHandle, egui};

use super::EditorUi;

use crate::{data::Atlas, ui::windows};

pub fn editor_ui(
    mut contexts: EguiContexts,
    mut editor_ui: ResMut<EditorUi>,
    mut atlas: ResMut<Atlas>,
    mut images: ResMut<Assets<Image>>,
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
            toolbar(ui, &mut editor_ui, &mut atlas);
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

    atlas_view(&mut root_ui, &mut editor_ui, &mut atlas);

    windows::import::import_window(ctx, &mut editor_ui, &mut atlas, &mut images);

    Ok(())
}
