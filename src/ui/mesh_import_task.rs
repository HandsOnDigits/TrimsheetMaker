use std::{
    path::PathBuf,
    sync::{
        Mutex,
        mpsc::{Receiver, TryRecvError},
    },
    thread,
};

use bevy::prelude::*;

use crate::{
    mesh::{
        MeshSceneState,
        import::{self, MeshBackend, MeshImportSettings, MeshScene},
        spawn_mesh_scene_with_visibility,
    },
    ui::{EditorUi, loading::LoadingState},
    viewport::MeshPreview,
};

type ImportResult = (PathBuf, Result<MeshScene, String>);

#[derive(Resource, Default)]
pub struct MeshImportTask {
    pub receiver: Option<Mutex<Receiver<ImportResult>>>,
    pub total: usize,
    pub completed: usize,
    pub imported: usize,
    pub errors: Vec<String>,
}

impl MeshImportTask {
    pub fn is_running(&self) -> bool {
        self.receiver.is_some()
    }

    pub fn start(
        &mut self,
        paths: Vec<PathBuf>,
        backend: MeshBackend,
        settings: MeshImportSettings,
        loading: &mut LoadingState,
    ) -> bool {
        if self.is_running() || paths.is_empty() {
            return false;
        }

        let (sender, receiver) = std::sync::mpsc::channel::<ImportResult>();

        self.total = paths.len();
        self.completed = 0;
        self.imported = 0;
        self.errors.clear();
        self.receiver = Some(Mutex::new(receiver));

        loading.start(format!("Importing {} mesh file(s)...", self.total));

        thread::spawn(move || {
            for path in paths {
                let result =
                    import::import(&path, backend, &settings).map_err(|error| error.to_string());

                if sender.send((path, result)).is_err() {
                    break;
                }
            }
        });

        true
    }
}

pub fn poll_mesh_import_task(
    mut task: ResMut<MeshImportTask>,
    mut loading: ResMut<LoadingState>,
    mut editor_ui: ResMut<EditorUi>,
    mut mesh_scene_state: ResMut<MeshSceneState>,
    mut mesh_preview: ResMut<MeshPreview>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    // Drain the channel under the lock, then process results without holding it.
    let (results, disconnected) = {
        let Some(receiver) = task.receiver.as_ref() else {
            return;
        };

        let Ok(receiver) = receiver.lock() else {
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

    for (path, result) in results {
        task.completed += 1;

        match result {
            Ok(scene) => {
                let mesh_id = mesh_scene_state.next_id;

                let (root, parts, bounds_center, bounds_radius) = spawn_mesh_scene_with_visibility(
                    &mut commands,
                    &mut meshes,
                    &scene,
                    mesh_id,
                    mesh_preview.material.clone(),
                    None,
                );

                let name = path
                    .file_stem()
                    .and_then(|name| name.to_str())
                    .unwrap_or("Mesh")
                    .to_owned();

                mesh_scene_state.add(name, path, scene, root, parts, bounds_center, bounds_radius);

                mesh_preview.selected_mesh_id = Some(mesh_id);
                task.imported += 1;
            }
            Err(error) => {
                task.errors.push(format!("{}: {error}", path.display()));
            }
        }

        loading.set_progress(task.completed as f32 / task.total.max(1) as f32);
    }

    if !disconnected {
        return;
    }

    if task.imported > 0 {
        editor_ui.project_dirty = true;
        mesh_preview.reset_view();
    }

    if task.errors.is_empty() {
        editor_ui.mesh_import_error = None;
        editor_ui.show_import_mesh_window = false;
    } else {
        editor_ui.mesh_import_error = Some(format!(
            "Imported {} mesh{}.\n\n{}",
            task.imported,
            if task.imported == 1 { "" } else { "es" },
            task.errors.join("\n")
        ));
        editor_ui.show_import_mesh_window = true;
    }

    task.receiver = None;
    task.total = 0;
    task.completed = 0;
    task.imported = 0;
    task.errors.clear();
    loading.finish();
}
