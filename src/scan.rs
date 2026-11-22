use crate::{
    config::Config,
    model::{LinkStatus, Model},
};
use same_file::is_same_file as paths_are_same;
use std::{
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

fn read_dir_ok(path: &Path) -> impl Iterator<Item = fs::DirEntry> {
    fs::read_dir(path).into_iter().flatten().flatten()
}

fn determine_status(real: &Path, target: &Path) -> LinkStatus {
    if !target.exists() {
        return LinkStatus::NotLinked;
    }
    // An existing file we cannot prove is our link counts as a conflict.
    match paths_are_same(real, target) {
        Ok(true) => LinkStatus::Linked,
        Ok(false) | Err(_) => LinkStatus::Conflict,
    }
}

/// HF canonical: `<model>/refs/main` holds the current revision hash.
/// Fallback: most recently modified snapshot directory.
fn current_snapshot(snapshots: &Path) -> Option<PathBuf> {
    if !snapshots.is_dir() {
        return None;
    }
    if let Some(model_dir) = snapshots.parent()
        && let Ok(rev) = fs::read_to_string(model_dir.join("refs/main"))
    {
        let candidate = snapshots.join(rev.trim());
        if candidate.is_dir() {
            return Some(candidate);
        }
    }
    let mut best: Option<(SystemTime, PathBuf)> = None;
    for entry in read_dir_ok(snapshots) {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Ok(mtime) = entry.metadata().and_then(|m| m.modified()) else {
            continue;
        };
        if best.as_ref().is_none_or(|(t, _)| mtime > *t) {
            best = Some((mtime, path));
        }
    }
    best.map(|(_, p)| p)
}

pub(crate) fn find_models(config: &Config) -> Vec<Model> {
    let cache_dir = &config.hf_cache;
    let lm_dir = &config.lmstudio_models;
    let mut found = Vec::new();

    if !cache_dir.is_dir() {
        return found;
    }

    for entry in read_dir_ok(cache_dir) {
        let model_folder = entry.path();
        let Some(name) = model_folder.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let Some(rest) = name.strip_prefix("models--") else {
            continue;
        };
        let Some((org, model)) = rest.split_once("--") else {
            continue;
        };
        if org.is_empty() || model.is_empty() {
            continue;
        }

        let Some(snapshot) = current_snapshot(&model_folder.join("snapshots")) else {
            continue;
        };

        for file_entry in read_dir_ok(&snapshot) {
            let entry_path = file_entry.path();
            let is_gguf = entry_path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case("gguf"));
            if !is_gguf {
                continue;
            }

            let Some(file_name) = entry_path.file_name() else {
                continue;
            };
            let Ok(real_file) = entry_path.canonicalize() else {
                continue;
            };
            let Ok(real_meta) = fs::metadata(&real_file) else {
                continue;
            };
            if !real_meta.is_file() || real_meta.len() == 0 {
                continue;
            }

            let size = real_meta.len();
            let target_file = lm_dir.join(org).join(model).join(file_name);
            let status = determine_status(&real_file, &target_file);
            let display = target_file
                .strip_prefix(lm_dir)
                .unwrap_or(&target_file)
                .display()
                .to_string();

            found.push(Model {
                display,
                real_file,
                target_file,
                size,
                status,
            });
        }
    }

    found.sort_by(|a, b| a.target_file.cmp(&b.target_file));
    found.dedup_by(|a, b| a.target_file == b.target_file);

    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn make_config(tmp: &Path) -> Config {
        Config {
            hf_cache: tmp.join("hub"),
            lmstudio_models: tmp.join("lm"),
        }
    }

    #[test]
    fn finds_gguf_files_in_hf_layout() {
        let tmp = tempdir().unwrap();
        let cfg = make_config(tmp.path());
        let snap = cfg
            .hf_cache
            .join("models--org--model")
            .join("snapshots")
            .join("abc123");
        fs::create_dir_all(&snap).unwrap();
        fs::write(snap.join("model.gguf"), b"fake weights").unwrap();
        fs::write(snap.join("config.json"), b"{}").unwrap();

        let models = find_models(&cfg);
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].status, LinkStatus::NotLinked);
        assert!(models[0].target_file.ends_with("org/model/model.gguf"));
    }

    #[test]
    fn detects_linked_and_conflict() {
        let tmp = tempdir().unwrap();
        let cfg = make_config(tmp.path());
        let snap = cfg
            .hf_cache
            .join("models--org--model")
            .join("snapshots")
            .join("abc123");
        fs::create_dir_all(&snap).unwrap();
        let real = snap.join("model.gguf");
        fs::write(&real, b"fake weights").unwrap();

        let target = cfg.lmstudio_models.join("org/model/model.gguf");
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::hard_link(&real, &target).unwrap();

        let models = find_models(&cfg);
        assert_eq!(models[0].status, LinkStatus::Linked);

        fs::remove_file(&target).unwrap();
        fs::write(&target, b"different content").unwrap();
        let models = find_models(&cfg);
        assert_eq!(models[0].status, LinkStatus::Conflict);
    }

    #[test]
    fn prefers_refs_main_snapshot() {
        let tmp = tempdir().unwrap();
        let model_dir = tmp.path().join("models--org--model");
        let snapshots = model_dir.join("snapshots");
        fs::create_dir_all(snapshots.join("old")).unwrap();
        fs::create_dir_all(snapshots.join("new")).unwrap();
        fs::create_dir_all(model_dir.join("refs")).unwrap();
        fs::write(model_dir.join("refs/main"), "old\n").unwrap();

        let chosen = current_snapshot(&snapshots).unwrap();
        assert!(chosen.ends_with("old"));
    }
}
