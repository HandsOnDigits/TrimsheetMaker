use std::{
    collections::{HashMap, HashSet},
    io,
    path::{Path, PathBuf},
};

use crate::data::{MaterialSource, MaterialTexture, TextureGroup, TextureNaming};

pub struct MaterialScan {
    pub materials: HashMap<String, MaterialSource>,
    pub naming: TextureNaming,
}

pub fn scan(root: &Path) -> io::Result<MaterialScan> {
    let mut folders = Vec::new();

    for entry in std::fs::read_dir(root)? {
        let entry = entry?;

        if entry.file_type()?.is_dir() {
            folders.push(entry.path());
        }
    }

    folders.sort();

    let naming = detect_naming(&folders)?;

    let mut materials = HashMap::new();

    for folder in &folders {
        let name = folder
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Unnamed")
            .to_owned();

        let mut textures = Vec::new();

        for entry in std::fs::read_dir(folder)? {
            let entry = entry?;

            if !entry.file_type()?.is_file() {
                continue;
            }

            let path = entry.path();

            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };

            let Some(group) = naming.group_for_filename(stem) else {
                continue;
            };

            textures.push(MaterialTexture {
                group,
                path,
                image: None,
            });
        }

        materials.insert(name.clone(), MaterialSource { name, textures });
    }

    Ok(MaterialScan { materials, naming })
}

fn detect_naming(folders: &[PathBuf]) -> io::Result<TextureNaming> {
    let mut suffix_folders: HashMap<String, HashSet<PathBuf>> = HashMap::new();

    for folder in folders {
        let folder_tokens = tokenize(
            folder
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default(),
        );

        for entry in std::fs::read_dir(folder)? {
            let entry = entry?;

            if !entry.file_type()?.is_file() {
                continue;
            }

            let path = entry.path();

            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };

            let tokens = tokenize(stem);

            let Some(suffix_start) = material_prefix_end(&tokens, &folder_tokens) else {
                continue;
            };

            if suffix_start >= tokens.len() {
                continue;
            }

            let suffix = tokens[suffix_start..].join("_");

            suffix_folders
                .entry(suffix)
                .or_default()
                .insert(folder.clone());
        }
    }

    let mut suffixes: Vec<(String, usize)> = suffix_folders
        .into_iter()
        .map(|(suffix, folders)| {
            let count = folders.len();
            (suffix, count)
        })
        .collect();

    // Most widely used suffixes first.
    //
    // Longer suffixes win when they have the same support.
    suffixes.sort_by(|a, b| {
        b.1.cmp(&a.1)
            .then_with(|| b.0.matches('_').count().cmp(&a.0.matches('_').count()))
            .then_with(|| b.0.len().cmp(&a.0.len()))
    });

    let mut groups = Vec::new();

    for (suffix, _) in suffixes {
        // Don't add `_color` if `_base_color` has already
        // been identified as the more specific suffix.
        if groups.iter().any(|group: &TextureGroup| {
            group
                .suffixes
                .iter()
                .any(|existing| existing.ends_with(&format!("_{suffix}")))
        }) {
            continue;
        }

        groups.push(TextureGroup {
            name: display_name(&suffix),
            suffixes: vec![format!("_{suffix}")],
        });
    }

    Ok(TextureNaming { groups })
}

fn material_prefix_end(filename: &[String], material: &[String]) -> Option<usize> {
    if material.is_empty() || filename.len() < material.len() {
        return None;
    }

    for start in 0..=filename.len() - material.len() {
        if filename[start..start + material.len()] == *material {
            return Some(start + material.len());
        }
    }

    None
}

fn tokenize(value: &str) -> Vec<String> {
    value
        .split(|c: char| !c.is_alphanumeric())
        .filter(|part| !part.is_empty())
        .map(|part| part.to_lowercase())
        .collect()
}

fn display_name(suffix: &str) -> String {
    suffix
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();

            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
