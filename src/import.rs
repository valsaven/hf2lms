use crate::model::{LinkStatus, Model};
use std::{fs, io};

#[derive(Default)]
pub(crate) struct ImportSummary {
    pub(crate) ok: usize,
    pub(crate) skipped_linked: usize,
    pub(crate) skipped_conflict: usize,
    pub(crate) failed: Vec<(String, String)>,
}

fn is_cross_device(err: &io::Error) -> bool {
    // Unix: EXDEV = 18; Windows: ERROR_NOT_SAME_DEVICE = 17
    matches!(err.raw_os_error(), Some(18) | Some(17))
}

pub(crate) fn import_selected(models: &[Model], indices: &[usize], force: bool) -> ImportSummary {
    let mut summary = ImportSummary::default();

    for &i in indices {
        let model = &models[i];

        match model.status {
            LinkStatus::Linked => {
                summary.skipped_linked += 1;
                continue;
            }
            LinkStatus::Conflict if !force => {
                summary.skipped_conflict += 1;
                continue;
            }
            _ => {}
        }

        if let Some(parent) = model.target_file.parent()
            && let Err(e) = fs::create_dir_all(parent)
        {
            summary
                .failed
                .push((model.display.clone(), format!("create dir: {e}")));
            continue;
        }

        if model.target_file.exists() {
            if !force {
                summary.failed.push((
                    model.display.clone(),
                    "target exists and is not our link (use --force)".into(),
                ));
                continue;
            }
            if let Err(e) = fs::remove_file(&model.target_file) {
                summary
                    .failed
                    .push((model.display.clone(), format!("remove target: {e}")));
                continue;
            }
        }

        match fs::hard_link(&model.real_file, &model.target_file) {
            Ok(_) => summary.ok += 1,
            Err(e) if is_cross_device(&e) => summary.failed.push((
                model.display.clone(),
                "cache and models on different filesystems; hard link impossible".into(),
            )),
            Err(e) => summary.failed.push((model.display.clone(), e.to_string())),
        }
    }

    summary
}
