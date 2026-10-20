use bevy::prelude::*;
use bevy_egui::egui;

#[derive(Resource, Default)]
pub struct LoadingState {
    pub message: Option<String>,
    pub progress: Option<f32>,
}

impl LoadingState {
    pub fn start(&mut self, message: impl Into<String>) {
        self.message = Some(message.into());
        self.progress = None;
    }

    pub fn set_progress(&mut self, progress: f32) {
        self.progress = Some(progress.clamp(0.0, 1.0));
    }

    pub fn finish(&mut self) {
        self.message = None;
        self.progress = None;
    }

    pub fn is_loading(&self) -> bool {
        self.message.is_some()
    }
}

pub fn loading_overlay(ctx: &egui::Context, loading: &LoadingState) {
    let Some(message) = loading.message.as_deref() else {
        return;
    };

    let screen_rect = ctx.viewport_rect();
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("loading_overlay_layer"),
    ));

    painter.rect_filled(screen_rect, 0.0, egui::Color32::from_black_alpha(160));

    let size = egui::vec2(320.0, 110.0);
    let position = screen_rect.center() - size * 0.5;

    egui::Area::new(egui::Id::new("loading_overlay"))
        .order(egui::Order::Tooltip)
        .fixed_pos(position)
        .interactable(true)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style())
                .inner_margin(egui::Margin::same(20))
                .show(ui, |ui| {
                    ui.set_min_width(280.0);

                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.heading(message);
                    });

                    ui.add_space(12.0);

                    if let Some(progress) = loading.progress {
                        ui.add(
                            egui::ProgressBar::new(progress)
                                .show_percentage()
                                .animate(true),
                        );
                    } else {
                        ui.add(egui::ProgressBar::new(0.0).animate(true));
                    }
                });
        });
}
