use std::path::PathBuf;

pub fn assimp_library_names() -> &'static [&'static str] {
    &["assimp.dll"]
}

pub fn default_assimp_library_paths() -> Vec<PathBuf> {
    Vec::new()
}
