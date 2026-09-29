pub mod editor_settings;
pub mod import_mesh;
pub mod import_multiple_materials;
pub mod import_single_material;
pub mod importer;
pub mod meshes;
pub mod rendering;

use bevy_egui::egui;

pub trait FloatingWindowInteraction {
    fn owns_pointer(&self, ctx: &egui::Context) -> bool;
}

impl<T> FloatingWindowInteraction for egui::InnerResponse<T> {
    fn owns_pointer(&self, ctx: &egui::Context) -> bool {
        let Some(pointer_pos) = ctx.pointer_hover_pos() else {
            return false;
        };

        self.response.rect.contains(pointer_pos)
    }
}
