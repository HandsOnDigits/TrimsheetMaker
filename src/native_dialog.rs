use bevy::prelude::*;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::project::PROJECT_EXTENSION;

#[derive(Debug, Clone, Copy)]
pub enum NativeDialogRequest {
    OpenProject,
    SaveProjectAs,
    ExportModel,
}

#[derive(Debug)]
pub enum NativeDialogResult {
    OpenProject(Option<PathBuf>),
    SaveProjectAs(Option<PathBuf>),
    ExportModel(Option<PathBuf>),
}

#[derive(Resource)]
pub struct NativeDialogState {
    pub active: bool,
    result: Arc<Mutex<Option<NativeDialogResult>>>,
}

impl Default for NativeDialogState {
    fn default() -> Self {
        Self {
            active: false,
            result: Arc::new(Mutex::new(None)),
        }
    }
}

impl NativeDialogState {
    pub fn start(&mut self, request: NativeDialogRequest) {
        // Do not allow multiple native dialogs to overlap.
        if self.active {
            return;
        }

        self.active = true;

        let result_slot = Arc::clone(&self.result);

        std::thread::spawn(move || {
            let result = match request {
                NativeDialogRequest::OpenProject => {
                    let path = rfd::FileDialog::new()
                        .add_filter("Trimsheet Project", &[PROJECT_EXTENSION])
                        .pick_file();

                    NativeDialogResult::OpenProject(path)
                }

                NativeDialogRequest::SaveProjectAs => {
                    let path = rfd::FileDialog::new()
                        .add_filter("Trimsheet Project", &[PROJECT_EXTENSION])
                        .set_file_name("Untitled.tsm")
                        .save_file()
                        .map(|mut path| {
                            if path.extension().is_none() {
                                path.set_extension(PROJECT_EXTENSION);
                            }

                            path
                        });

                    NativeDialogResult::SaveProjectAs(path)
                }

                NativeDialogRequest::ExportModel => {
                    let path = rfd::FileDialog::new()
                        .add_filter("glTF Binary", &["glb"])
                        .set_file_name("atlas_model.glb")
                        .save_file();

                    NativeDialogResult::ExportModel(path)
                }
            };

            if let Ok(mut slot) = result_slot.lock() {
                *slot = Some(result);
            }
        });
    }

    /// Called on the main thread to retrieve a completed dialog result.
    pub fn take_result(&mut self) -> Option<NativeDialogResult> {
        let result = self.result.lock().ok()?.take();

        if result.is_some() {
            self.active = false;
        }

        result
    }
}
