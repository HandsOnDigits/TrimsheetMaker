use std::path::Path;

use super::{MeshImportError, MeshScene};

pub fn import(_path: &Path) -> Result<MeshScene, MeshImportError> {
    Err(MeshImportError::Other(
        "glTF importer is not implemented yet".to_owned(),
    ))
}
