use std::path::PathBuf;

pub fn assimp_library_names() -> &'static [&'static str] {
    &["libassimp.so", "libassimp.so.6"]
}

pub fn default_assimp_library_paths() -> Vec<PathBuf> {
    Vec::new()
}
