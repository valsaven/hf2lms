use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LinkStatus {
    NotLinked,
    Linked,
    Conflict,
}

#[derive(Debug)]
pub(crate) struct Model {
    pub(crate) display: String,
    pub(crate) real_file: PathBuf,
    pub(crate) target_file: PathBuf,
    pub(crate) size: u64,
    pub(crate) status: LinkStatus,
}

pub(crate) fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_size_works() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(1023), "1023 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(4 * 1024 * 1024 * 1024), "4.0 GB");
    }
}
