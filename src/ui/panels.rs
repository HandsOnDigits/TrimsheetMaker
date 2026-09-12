use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use super::{Atlas, AtlasMaterial, EditorUi};

pub fn editor_ui(
    mut contexts: EguiContexts,
    mut editor_ui: ResMut<EditorUi>,
    mut atlas: ResMut<Atlas>,
) -> Result {
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

    Ok(())
}

// -----------------------------------------------------------------------------
// Toolbar
// -----------------------------------------------------------------------------

fn toolbar(ui: &mut egui::Ui, editor_ui: &mut EditorUi, atlas: &mut Atlas) {
    ui.horizontal(|ui| {
        ui.add_space(8.0);

        ui.menu_button("File", |ui| {
            if ui.button("New Atlas").clicked() {
                *atlas = Atlas::default();

                editor_ui.selected_material = Some(0);
                editor_ui.atlas_zoom = 1.0;
                editor_ui.atlas_pan = egui::Vec2::ZERO;

                ui.close();
            }

            if ui.button("Open Project…").clicked() {
                ui.close();
            }

            if ui.button("Save Project").clicked() {
                ui.close();
            }

            ui.separator();

            if ui.button("Export Atlas…").clicked() {
                ui.close();
            }
        });

        ui.menu_button("Edit", |ui| {
            if ui.button("Add Material").clicked() {
                let index = atlas.add_placeholder();
                editor_ui.selected_material = Some(index);

                ui.close();
            }

            if ui.button("Remove Selected").clicked() {
                if let Some(index) = editor_ui.selected_material {
                    atlas.remove(index);
                    editor_ui.selected_material = None;
                }

                ui.close();
            }
        });

        ui.separator();

        if ui
            .add_sized([130.0, 30.0], egui::Button::new("＋ Add Material"))
            .clicked()
        {
            let index = atlas.add_placeholder();
            editor_ui.selected_material = Some(index);
        }

        if ui
            .add_sized([100.0, 30.0], egui::Button::new("Pack Atlas"))
            .clicked()
        {
            // Real atlas packing goes here.
        }

        if ui
            .add_sized([90.0, 30.0], egui::Button::new("Export"))
            .clicked()
        {
            // Export implementation goes here.
        }

        ui.separator();

        ui.menu_button("View", |ui| {
            ui.checkbox(&mut editor_ui.show_materials, "Materials");

            ui.checkbox(&mut editor_ui.show_inspector, "Inspector");

            ui.checkbox(&mut editor_ui.show_status, "Status Bar");

            ui.separator();

            if ui.button("Reset Atlas View").clicked() {
                editor_ui.atlas_zoom = 1.0;
                editor_ui.atlas_pan = egui::Vec2::ZERO;

                ui.close();
            }
        });

        ui.separator();

        ui.label(format!("Atlas: {} × {}", atlas.width, atlas.height));
    });
}

// -----------------------------------------------------------------------------
// Materials
// -----------------------------------------------------------------------------

fn materials_panel(ui: &mut egui::Ui, editor_ui: &mut EditorUi, atlas: &mut Atlas) {
    ui.heading("Materials");

    ui.horizontal(|ui| {
        ui.label(format!("{} materials", atlas.materials.len()));
    });

    ui.separator();

    egui::ScrollArea::vertical()
        .id_salt("materials_scroll")
        .show(ui, |ui| {
            // Avoid borrowing `editor_ui` mutably while iterating the atlas.
            let mut clicked = None;

            for material in &atlas.materials {
                let selected = editor_ui.selected_material == Some(material.index);

                if material_row(ui, material, selected).clicked() {
                    clicked = Some(material.index);
                }
            }

            if let Some(index) = clicked {
                editor_ui.selected_material = Some(index);
            }
        });

    ui.separator();

    if ui
        .add_sized(
            [ui.available_width(), 32.0],
            egui::Button::new("＋ Add Material"),
        )
        .clicked()
    {
        let index = atlas.add_placeholder();
        editor_ui.selected_material = Some(index);
    }
}

fn material_row(ui: &mut egui::Ui, material: &AtlasMaterial, selected: bool) -> egui::Response {
    let height = 48.0;
    let width = ui.available_width();

    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::click());

    if selected {
        ui.painter()
            .rect_filled(rect, 4.0, ui.visuals().selection.bg_fill);
    } else if response.hovered() {
        ui.painter()
            .rect_filled(rect, 4.0, ui.visuals().widgets.hovered.bg_fill);
    }

    let thumbnail_rect = egui::Rect::from_min_size(
        rect.left_top() + egui::vec2(6.0, 6.0),
        egui::vec2(36.0, 36.0),
    );

    ui.painter()
        .rect_filled(thumbnail_rect, 3.0, egui::Color32::from_gray(55));

    ui.painter().text(
        thumbnail_rect.center(),
        egui::Align2::CENTER_CENTER,
        material.index.to_string(),
        egui::FontId::proportional(14.0),
        egui::Color32::WHITE,
    );

    let text_pos = rect.left_top() + egui::vec2(52.0, 7.0);

    ui.painter().text(
        text_pos,
        egui::Align2::LEFT_TOP,
        &material.name,
        egui::FontId::proportional(14.0),
        ui.visuals().text_color(),
    );

    ui.painter().text(
        text_pos + egui::vec2(0.0, 19.0),
        egui::Align2::LEFT_TOP,
        format!("#{}  {}", material.index, material.source),
        egui::FontId::proportional(11.0),
        ui.visuals().weak_text_color(),
    );

    response
}

// -----------------------------------------------------------------------------
// Inspector
// -----------------------------------------------------------------------------

fn inspector_panel(ui: &mut egui::Ui, editor_ui: &mut EditorUi, atlas: &mut Atlas) {
    ui.heading("Inspector");

    ui.separator();

    let Some(index) = editor_ui.selected_material else {
        ui.vertical_centered(|ui| {
            ui.add_space(16.0);

            ui.label(egui::RichText::new("Nothing selected").weak().size(15.0));

            ui.add_space(4.0);

            ui.label(egui::RichText::new("Select a material from the list.").weak());
        });

        return;
    };

    let Some(material) = atlas.selected_mut(Some(index)) else {
        editor_ui.selected_material = None;
        return;
    };

    ui.label(egui::RichText::new(format!("Material #{}", material.index)).strong());

    ui.add_space(8.0);

    egui::Grid::new("material_properties")
        .num_columns(2)
        .spacing([12.0, 8.0])
        .show(ui, |ui| {
            ui.label("Name");
            ui.text_edit_singleline(&mut material.name);
            ui.end_row();

            ui.label("Source");
            ui.label(&material.source);
            ui.end_row();

            ui.separator();
            ui.separator();
            ui.end_row();

            ui.label("Position");

            ui.horizontal(|ui| {
                ui.label("X");
                ui.add(egui::DragValue::new(&mut material.x).speed(1));

                ui.label("Y");
                ui.add(egui::DragValue::new(&mut material.y).speed(1));
            });

            ui.end_row();

            ui.label("Size");

            ui.horizontal(|ui| {
                ui.label("W");
                ui.add(egui::DragValue::new(&mut material.width).speed(1));

                ui.label("H");
                ui.add(egui::DragValue::new(&mut material.height).speed(1));
            });

            ui.end_row();
        });

    ui.add_space(16.0);

    ui.separator();

    ui.label(egui::RichText::new("Texture").strong());

    ui.add_space(4.0);

    ui.label(egui::RichText::new(&material.source).weak());

    ui.add_space(8.0);

    let _ = ui.button("Replace Texture…");
}

// -----------------------------------------------------------------------------
// Atlas viewport
// -----------------------------------------------------------------------------

fn atlas_view(ui: &mut egui::Ui, editor_ui: &mut EditorUi, atlas: &mut Atlas) {
    ui.horizontal(|ui| {
        ui.heading("Atlas");

        ui.separator();

        ui.label(format!("{} × {} px", atlas.width, atlas.height));

        ui.separator();

        if ui.button("−").clicked() {
            editor_ui.atlas_zoom = (editor_ui.atlas_zoom * 0.9).max(0.1);
        }

        ui.label(format!("{:.0}%", editor_ui.atlas_zoom * 100.0));

        if ui.button("+").clicked() {
            editor_ui.atlas_zoom = (editor_ui.atlas_zoom * 1.1).min(10.0);
        }

        if ui.button("Reset").clicked() {
            editor_ui.atlas_zoom = 1.0;
            editor_ui.atlas_pan = egui::Vec2::ZERO;
        }
    });

    ui.separator();

    let available = ui.available_size();

    let (viewport_rect, response) =
        ui.allocate_exact_size(available, egui::Sense::click_and_drag());

    if response.dragged_by(egui::PointerButton::Middle)
        || response.dragged_by(egui::PointerButton::Secondary)
    {
        editor_ui.atlas_pan += response.drag_delta();
    }

    let painter = ui.painter_at(viewport_rect);

    draw_atlas(&painter, viewport_rect, editor_ui, atlas);

    if response.clicked() {
        if let Some(pointer) = response.interact_pointer_pos() {
            editor_ui.selected_material =
                atlas_material_at_position(pointer, viewport_rect, editor_ui, atlas);
        }
    }
}

fn draw_atlas(painter: &egui::Painter, viewport: egui::Rect, editor_ui: &EditorUi, atlas: &Atlas) {
    let zoom = editor_ui.atlas_zoom;

    let atlas_size = egui::vec2(atlas.width as f32 * zoom, atlas.height as f32 * zoom);

    let center = viewport.center() + editor_ui.atlas_pan;

    let atlas_rect = egui::Rect::from_center_size(center, atlas_size);

    draw_checkerboard(painter, atlas_rect, zoom);

    painter.rect_stroke(
        atlas_rect,
        0.0,
        egui::Stroke::new(1.0, egui::Color32::from_gray(120)),
        egui::StrokeKind::Outside,
    );

    for material in &atlas.materials {
        let rect = atlas_material_rect(material, atlas_rect, zoom);

        let selected = editor_ui.selected_material == Some(material.index);

        draw_material_region(painter, rect, material, selected);
    }
}

fn draw_checkerboard(painter: &egui::Painter, rect: egui::Rect, zoom: f32) {
    let checker_size = (16.0 * zoom).max(4.0);

    let columns = (rect.width() / checker_size).ceil() as i32;

    let rows = (rect.height() / checker_size).ceil() as i32;

    let light = egui::Color32::from_gray(58);
    let dark = egui::Color32::from_gray(48);

    for y in 0..rows {
        for x in 0..columns {
            let min = egui::pos2(
                rect.left() + x as f32 * checker_size,
                rect.top() + y as f32 * checker_size,
            );

            let max = egui::pos2(min.x + checker_size, min.y + checker_size);

            let tile = egui::Rect::from_min_max(min, max).intersect(rect);

            let color = if (x + y) % 2 == 0 { light } else { dark };

            painter.rect_filled(tile, 0.0, color);
        }
    }
}

fn draw_material_region(
    painter: &egui::Painter,
    rect: egui::Rect,
    material: &AtlasMaterial,
    selected: bool,
) {
    let fill = if selected {
        egui::Color32::from_rgba_unmultiplied(80, 140, 255, 100)
    } else {
        egui::Color32::from_rgba_unmultiplied(120, 120, 120, 45)
    };

    painter.rect_filled(rect, 0.0, fill);

    let stroke = if selected {
        egui::Stroke::new(2.0, egui::Color32::from_rgb(80, 170, 255))
    } else {
        egui::Stroke::new(1.0, egui::Color32::from_rgb(170, 170, 170))
    };

    painter.rect_stroke(rect, 0.0, stroke, egui::StrokeKind::Inside);

    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        material.index.to_string(),
        egui::FontId::proportional(16.0),
        egui::Color32::WHITE,
    );
}

fn atlas_material_rect(material: &AtlasMaterial, atlas_rect: egui::Rect, zoom: f32) -> egui::Rect {
    let min = egui::pos2(
        atlas_rect.left() + material.x as f32 * zoom,
        atlas_rect.top() + material.y as f32 * zoom,
    );

    let size = egui::vec2(material.width as f32 * zoom, material.height as f32 * zoom);

    egui::Rect::from_min_size(min, size)
}

fn atlas_material_at_position(
    position: egui::Pos2,
    viewport: egui::Rect,
    editor_ui: &EditorUi,
    atlas: &Atlas,
) -> Option<usize> {
    let zoom = editor_ui.atlas_zoom;

    let atlas_size = egui::vec2(atlas.width as f32 * zoom, atlas.height as f32 * zoom);

    let center = viewport.center() + editor_ui.atlas_pan;

    let atlas_rect = egui::Rect::from_center_size(center, atlas_size);

    if !atlas_rect.contains(position) {
        return None;
    }

    let atlas_x = ((position.x - atlas_rect.left()) / zoom) as u32;

    let atlas_y = ((position.y - atlas_rect.top()) / zoom) as u32;

    atlas
        .materials
        .iter()
        .rev()
        .find(|material| {
            atlas_x >= material.x
                && atlas_x < material.x + material.width
                && atlas_y >= material.y
                && atlas_y < material.y + material.height
        })
        .map(|material| material.index)
}

// -----------------------------------------------------------------------------
// Status bar
// -----------------------------------------------------------------------------

fn status_bar(ui: &mut egui::Ui, editor_ui: &EditorUi, atlas: &Atlas) {
    ui.horizontal(|ui| {
        ui.add_space(8.0);

        ui.label(format!("Atlas {} × {}", atlas.width, atlas.height));

        ui.separator();

        ui.label(format!("{} materials", atlas.materials.len()));

        if let Some(index) = editor_ui.selected_material {
            ui.separator();

            ui.label(format!("Selected: #{}", index));
        }

        ui.separator();

        ui.label(egui::RichText::new("Ready").weak());

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(format!("Zoom {:.0}%", editor_ui.atlas_zoom * 100.0));
        });
    });
}
