use std::{
    collections::{HashMap, HashSet},
    io,
    path::{Path, PathBuf},
};

use crate::data::{TextureGroup, TextureNaming};

pub fn detect_naming(folders: &[PathBuf]) -> io::Result<TextureNaming> {
    let mut suffix_folders: HashMap<String, HashSet<PathBuf>> = HashMap::new();

    for folder in folders {
        let folder_tokens = tokenize(
            folder
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default(),
        );

        let mut folder_suffixes = Vec::new();

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

            let Some(prefix_end) = material_prefix_end(&tokens, &folder_tokens) else {
                continue;
            };

            if prefix_end >= tokens.len() {
                continue;
            }

            folder_suffixes.push(tokens[prefix_end..].to_vec());
        }

        if folder_suffixes.is_empty() {
            continue;
        }

        let common_prefix_len = common_prefix_len(&folder_suffixes);

        for suffix_tokens in folder_suffixes {
            if common_prefix_len >= suffix_tokens.len() {
                continue;
            }

            let suffix = suffix_tokens[common_prefix_len..].join("_");

            suffix_folders
                .entry(suffix)
                .or_default()
                .insert(folder.clone());
        }
    }

    let mut suffixes: Vec<(String, usize)> = suffix_folders
        .into_iter()
        .map(|(suffix, folders)| (suffix, folders.len()))
        .collect();

    suffixes.sort_by(|a, b| {
        b.1.cmp(&a.1)
            .then_with(|| b.0.matches('_').count().cmp(&a.0.matches('_').count()))
            .then_with(|| b.0.len().cmp(&a.0.len()))
    });

    let mut groups = Vec::new();

    for (suffix, _) in suffixes {
        if groups.iter().any(|group: &TextureGroup| {
            group
                .suffixes
                .iter()
                .any(|existing| existing.ends_with(&format!("_{suffix}")))
        }) {
            continue;
        }

        groups.push(TextureGroup {
            id: uuid::Uuid::new_v4().to_string(),
            name: display_name(&suffix),
            suffixes: vec![format!("_{suffix}")],
        });
    }

    Ok(TextureNaming { groups })
}

pub fn collect_texture_paths(folder: &Path) -> io::Result<Vec<PathBuf>> {
    let mut textures = Vec::new();

    for entry in std::fs::read_dir(folder)? {
        let entry = entry?;

        if !entry.file_type()?.is_file() {
            continue;
        }

        let path = entry.path();

        if path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("exr"))
        {
            textures.push(path);
        }
    }

    textures.sort();

    Ok(textures)
}

fn common_prefix_len(tokens: &[Vec<String>]) -> usize {
    let Some(first) = tokens.first() else {
        return 0;
    };

    let mut length = first.len();

    for other in &tokens[1..] {
        length = length.min(other.len());

        for index in 0..length {
            if first[index] != other[index] {
                length = index;
                break;
            }
        }
    }

    length
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
