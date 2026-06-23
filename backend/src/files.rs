use crate::mock_data::FilesState;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

pub fn list_files_state(root: &Path, subpath: Option<&str>) -> Result<FilesState, String> {
    let directory = trusted_path(root, subpath.unwrap_or(""))?;
    if !directory.is_dir() {
        return Err("File browser path must be a folder".into());
    }

    let mut rows = Vec::new();
    for entry in fs::read_dir(&directory).map_err(|error| format!("Cannot list files: {error}"))? {
        let entry = entry.map_err(|error| format!("Cannot read file entry: {error}"))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name == ".git" {
            continue;
        }
        let kind = entry
            .file_type()
            .map_err(|error| format!("Cannot inspect file entry: {error}"))?;
        let entry_path = match subpath.unwrap_or("").trim() {
            "" => name.clone(),
            base => format!("{base}/{name}"),
        };
        let (git_state, ignored_state) = file_metadata(root, &entry_path);
        let mut row = vec![
            name.clone(),
            if kind.is_dir() { "folder" } else { "file" }.into(),
            if kind.is_dir() {
                "file-explorer"
            } else {
                "file-editor"
            }
            .into(),
            entry_path,
            depth_for(subpath.unwrap_or("")).to_string(),
        ];
        row.push(git_state);
        row.push(ignored_state);
        rows.push(row);
    }
    rows.sort_by(|left, right| match (left[1].as_str(), right[1].as_str()) {
        ("folder", "file") => std::cmp::Ordering::Less,
        ("file", "folder") => std::cmp::Ordering::Greater,
        _ => left[0].cmp(&right[0]),
    });

    Ok(FilesState {
        roots: rows,
        breadcrumbs: breadcrumbs(root, subpath.unwrap_or("")),
        lines: Vec::new(),
        current_path: subpath.unwrap_or("").into(),
        content: String::new(),
        saved_content: String::new(),
        dirty: false,
        status: "Ready".into(),
    })
}

pub fn read_file_state(root: &Path, path: &str) -> Result<FilesState, String> {
    let file = trusted_path(root, path)?;
    if !file.is_file() {
        return Err("File path must be a regular file".into());
    }
    let content =
        fs::read_to_string(&file).map_err(|error| format!("Cannot read file: {error}"))?;
    Ok(file_state(root, path, &content, "Opened"))
}

pub fn save_file_state(root: &Path, path: &str, content: &str) -> Result<FilesState, String> {
    let file = trusted_path(root, path)?;
    if file.is_dir() {
        return Err("Cannot save over a folder".into());
    }
    if let Some(parent) = file.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Cannot create file folder: {error}"))?;
    }
    fs::write(&file, content).map_err(|error| format!("Cannot save file: {error}"))?;
    Ok(file_state(root, path, content, "Saved"))
}

pub fn trusted_path(root: &Path, requested: &str) -> Result<PathBuf, String> {
    let canonical_root =
        fs::canonicalize(root).map_err(|error| format!("Cannot resolve trusted root: {error}"))?;
    let relative = Path::new(requested);
    if relative.is_absolute() {
        return Err("Path is outside trusted root".into());
    }
    if relative
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err("Path is outside trusted root".into());
    }
    if relative
        .components()
        .any(|component| matches!(component, Component::Normal(name) if name == ".git"))
    {
        return Err("Casual .git mutation is rejected".into());
    }

    let candidate = canonical_root.join(relative);
    let canonical_candidate = if candidate.exists() {
        fs::canonicalize(&candidate)
            .map_err(|error| format!("Cannot resolve file path: {error}"))?
    } else {
        let parent = candidate
            .parent()
            .ok_or_else(|| "Path is outside trusted root".to_string())?;
        fs::canonicalize(parent)
            .map_err(|error| format!("Cannot resolve parent path: {error}"))?
            .join(candidate.file_name().unwrap_or_default())
    };

    if !canonical_candidate.starts_with(&canonical_root) {
        return Err("Path is outside trusted root".into());
    }
    Ok(candidate)
}

fn file_state(root: &Path, path: &str, content: &str, status: &str) -> FilesState {
    let mut state = list_files_state(root, None).unwrap_or(FilesState {
        roots: Vec::new(),
        breadcrumbs: breadcrumbs(root, path),
        lines: Vec::new(),
        current_path: path.into(),
        content: String::new(),
        saved_content: String::new(),
        dirty: false,
        status: String::new(),
    });
    state.breadcrumbs = breadcrumbs(root, path);
    state.lines = content.lines().map(String::from).collect();
    state.current_path = path.into();
    state.content = content.into();
    state.saved_content = content.into();
    state.dirty = false;
    state.status = status.into();
    state
}

fn depth_for(path: &str) -> usize {
    Path::new(path)
        .components()
        .filter(|component| matches!(component, Component::Normal(_)))
        .count()
}

fn file_metadata(root: &Path, path: &str) -> (String, String) {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("status")
        .arg("--porcelain=v1")
        .arg("--untracked-files=all")
        .arg("--")
        .arg(path)
        .output();

    let Ok(output) = output else {
        return (String::new(), String::new());
    };
    if !output.status.success() {
        return (String::new(), String::new());
    }
    let status = String::from_utf8_lossy(&output.stdout);
    let mut git_state = String::new();
    for line in status.lines() {
        let code = line.get(0..2).unwrap_or("");
        if code.contains('D') {
            git_state = "deleted".into();
            break;
        }
        if code.contains('A') || code.contains('?') {
            git_state = "added".into();
            continue;
        }
        if code.trim().is_empty() {
            continue;
        }
        if git_state.is_empty() {
            git_state = "modified".into();
        }
    }
    let ignored_state = exact_ignored_state(root, path);
    (git_state, ignored_state)
}

fn exact_ignored_state(root: &Path, path: &str) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("check-ignore")
        .arg("--quiet")
        .arg("--")
        .arg(path)
        .status();
    if output.is_ok_and(|status| status.success()) {
        "ignored".into()
    } else {
        String::new()
    }
}

fn breadcrumbs(root: &Path, path: &str) -> Vec<String> {
    let mut crumbs = vec![root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Workspace")
        .to_string()];
    crumbs.extend(
        Path::new(path)
            .components()
            .filter_map(|component| match component {
                Component::Normal(name) => name.to_str().map(String::from),
                _ => None,
            }),
    );
    crumbs
}
