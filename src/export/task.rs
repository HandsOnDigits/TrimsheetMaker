use bevy::prelude::*;

use std::{
    sync::{Mutex, mpsc},
    thread,
};

pub(crate) enum ExportMessage {
    Progress(f32),
    Finished(anyhow::Result<String>),
}

pub(crate) struct ActiveExport {
    message: String,
    progress: Option<f32>,
    receiver: Mutex<mpsc::Receiver<ExportMessage>>,
}

#[derive(Default)]
pub struct ExportTaskState {
    pub(crate) active: Option<ActiveExport>,
}

impl ExportTaskState {
    pub fn is_active(&self) -> bool {
        self.active.is_some()
    }

    pub fn start<F>(&mut self, message: impl Into<String>, job: F) -> bool
    where
        F: FnOnce(mpsc::Sender<ExportMessage>) -> anyhow::Result<String> + Send + 'static,
    {
        if self.is_active() {
            return false;
        }

        let (sender, receiver) = mpsc::channel();

        self.active = Some(ActiveExport {
            message: message.into(),
            progress: None,
            receiver: Mutex::new(receiver),
        });

        thread::spawn(move || {
            let result = job(sender.clone());
            let _ = sender.send(ExportMessage::Finished(result));
        });

        true
    }
}

/// Poll this on Bevy's main thread so the UI stays responsive.

pub fn poll_export_task(
    mut editor_ui: ResMut<crate::ui::EditorUi>,
    mut loading: ResMut<crate::ui::loading::LoadingState>,
) {
    // Collect messages in a separate scope so the receiver lock and task
    // borrow are dropped before we mutate the active task.
    let (message, messages, poisoned) = {
        let Some(task) = editor_ui.export_task.active.as_ref() else {
            return;
        };

        let message = task.message.clone();

        match task.receiver.lock() {
            Ok(receiver) => (message, receiver.try_iter().collect::<Vec<_>>(), false),
            Err(_) => (message, Vec::new(), true),
        }
    };

    if poisoned {
        error!("Export task channel was poisoned");
        editor_ui.export_task.active = None;
        loading.finish();
        return;
    }

    if loading.message.as_deref() != Some(message.as_str()) {
        loading.start(message);
    }

    let mut finished = false;

    for message in messages {
        match message {
            ExportMessage::Progress(progress) => {
                let progress = progress.clamp(0.0, 1.0);

                if let Some(task) = editor_ui.export_task.active.as_mut() {
                    task.progress = Some(progress);
                }

                loading.set_progress(progress);
            }
            ExportMessage::Finished(Ok(message)) => {
                info!("{message}");
                finished = true;
            }
            ExportMessage::Finished(Err(error)) => {
                error!("Export failed: {error:#}");
                finished = true;
            }
        }
    }

    if finished {
        editor_ui.export_task.active = None;
        loading.finish();
    }
}
