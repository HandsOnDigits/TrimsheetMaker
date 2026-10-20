use std::{
    path::PathBuf,
    sync::{
        Mutex,
        mpsc::{Receiver, TryRecvError},
    },
    thread,
};

use bevy::prelude::*;
use rayon::prelude::*;

use crate::{
    data::Atlas,
    ui::{EditorUi, loading::LoadingState},
};

type PendingTexture = (usize, String, PathBuf);
type TextureResult = (usize, String, PathBuf, Result<Image, String>);

#[derive(Resource, Default)]
pub struct ImageImportTask {
    receiver: Option<Mutex<Receiver<TextureResult>>>,
    pub total: usize,
    pub completed: usize,
    image_changed: bool,
}

impl ImageImportTask {
    pub fn is_running(&self) -> bool {
        self.receiver.is_some()
    }

    pub fn start(
        &mut self,
        atlas: &Atlas,
        material_ids: &[usize],
        loading: &mut LoadingState,
    ) -> bool {
        if self.is_running() {
            return false;
        }

        // Gather paths on the main thread. The worker receives owned data only.
        let pending: Vec<PendingTexture> = atlas
            .materials
            .iter()
            .filter(|material| material_ids.contains(&material.index))
            .flat_map(|material| {
                material
                    .textures
                    .iter()
                    .filter(|texture| texture.image.is_none())
                    .map(move |texture| (material.index, texture.id.clone(), texture.path.clone()))
            })
            .collect();

        if pending.is_empty() {
            return false;
        }

        let (sender, receiver) = std::sync::mpsc::channel::<TextureResult>();

        self.total = pending.len();
        self.completed = 0;
        self.image_changed = false;
        self.receiver = Some(Mutex::new(receiver));

        loading.start(format!("Loading {} texture(s)...", self.total));

        thread::spawn(move || {
            pending.into_par_iter().for_each_with(
                sender,
                |sender, (material_id, texture_id, path)| {
                    let result =
                        crate::image::import::load(&path).map_err(|error| error.to_string());

                    let _ = sender.send((material_id, texture_id, path, result));
                },
            );
        });

        true
    }
}

/// Poll from the existing UI system. This is deliberately a regular helper,
/// not a separate Bevy system, because that UI system already owns these
/// mutable resources.
pub fn poll_image_import_task(
    task: &mut ImageImportTask,
    loading: &mut LoadingState,
    editor_ui: &mut EditorUi,
    atlas: &mut Atlas,
    images: &mut Assets<Image>,
) {
    let (results, disconnected) = {
        let Some(receiver) = task.receiver.as_ref() else {
            return;
        };

        let Ok(receiver) = receiver.lock() else {
            error!("Image import result channel was poisoned.");
            return;
        };

        let mut results = Vec::new();
        let mut disconnected = false;

        loop {
            match receiver.try_recv() {
                Ok(result) => results.push(result),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    disconnected = true;
                    break;
                }
            }
        }

        (results, disconnected)
    };

    for (material_id, texture_id, path, result) in results {
        task.completed += 1;

        match result {
            Ok(image) => {
                // Find the target by stable material/texture identity.
                // If the material or texture was deleted while decoding,
                // discard the result instead of assigning it to another slot.
                if let Some(material) = atlas
                    .materials
                    .iter_mut()
                    .find(|material| material.index == material_id)
                {
                    if let Some(texture) = material.textures.iter_mut().find(|texture| {
                        texture.id == texture_id
                            && texture.path.as_path() == path.as_path()
                            && texture.image.is_none()
                    }) {
                        texture.image = Some(images.add(image));
                        task.image_changed = true;
                    }
                }
            }
            Err(error) => {
                error!("Failed to load texture '{}': {error}", path.display());
            }
        }

        loading.set_progress(task.completed as f32 / task.total.max(1) as f32);
    }

    // The worker has exited and all queued results have been drained.
    if !disconnected {
        return;
    }

    atlas.finish_texture_loading(images, task.image_changed);

    editor_ui.project_settings.sync_from_atlas(atlas);
    editor_ui.project_dirty = true;

    task.receiver = None;
    task.total = 0;
    task.completed = 0;
    task.image_changed = false;

    loading.finish();
}
