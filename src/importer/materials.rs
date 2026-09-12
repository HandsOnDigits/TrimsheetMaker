use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::data::{TextureGroup, TextureNaming};

pub struct MaterialSource {
    pub name: String,
    pub folder: PathBuf,
    pub textures: Vec<MaterialTextureSource>,
}

pub struct MaterialTextureSource {
    pub group: String,
    pub path: PathBuf,
}

pub struct MaterialScan {
    pub materials: Vec<MaterialSource>,
    pub naming: TextureNaming,
}

pub fn scan(root: &Path) -> std::io::Result<MaterialScan> {
    let mut folders = Vec::new();

    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            folders.push(path);
        }
    }

    let naming = detect_naming(&folders)?;

    let mut materials = Vec::new();

    for folder in folders {
        materials.push(scan_material(&folder, &naming)?);
    }

    materials.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    Ok(MaterialScan { materials, naming })
}

fn detect_naming(folders: &[PathBuf]) -> std::io::Result<TextureNaming> {
    let mut candidates: HashMap<String, usize> = HashMap::new();

    for folder in folders {
        for entry in std::fs::read_dir(folder)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            let Some(stem) = path.file_stem().and_then(|name| name.to_str()) else {
                continue;
            };

            for candidate in suffix_candidates(stem) {
                *candidates.entry(candidate).or_default() += 1;
            }
        }
    }

    // A suffix that occurs more than once across the material set
    // is a useful indication that it represents a texture type.
    let mut suffixes: Vec<(String, usize)> = candidates
        .into_iter()
        .filter(|(_, count)| *count >= 2)
        .collect();

    // Longest candidates first.
    suffixes.sort_by(|a, b| b.0.len().cmp(&a.0.len()).then_with(|| b.1.cmp(&a.1)));

    // Remove candidates that are just a shorter suffix of an already
    // recognized suffix.
    let mut recognized = Vec::new();

    for (suffix, _) in suffixes {
        if recognized
            .iter()
            .any(|existing: &String| existing.ends_with(&suffix))
        {
            continue;
        }

        recognized.push(suffix);
    }

    recognized.sort();

    let groups = recognized
        .into_iter()
        .map(|suffix| TextureGroup {
            name: display_group_name(&suffix),
            suffixes: vec![suffix],
        })
        .collect();

    Ok(TextureNaming { groups })
}

fn suffix_candidates(stem: &str) -> Vec<String> {
    let parts: Vec<&str> = stem.split('_').collect();

    let mut candidates = Vec::new();

    for start in 1..parts.len() {
        let suffix = format!("_{}", parts[start..].join("_"));
        candidates.push(suffix.to_lowercase());
    }

    candidates
}

fn display_group_name(suffix: &str) -> String {
    suffix
        .trim_start_matches('_')
        .split('_')
        .map(|part| {
            let mut chars = part.chars();

            match chars.next() {
                Some(first) => {
                    format!("{}{}", first.to_uppercase(), chars.as_str().to_lowercase())
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn scan_material(folder: &Path, naming: &TextureNaming) -> std::io::Result<MaterialSource> {
    let name = folder
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Material")
        .to_owned();

    let mut textures = Vec::new();

    for entry in std::fs::read_dir(folder)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let Some(stem) = path.file_stem().and_then(|name| name.to_str()) else {
            continue;
        };

        let Some(group) = naming.group_for_filename(stem) else {
            continue;
        };

        textures.push(MaterialTextureSource { group, path });
    }

    Ok(MaterialSource {
        name,
        folder: folder.to_owned(),
        textures,
    })
}
