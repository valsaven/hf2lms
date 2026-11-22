use anyhow::{Context, Result, bail};
use std::path::PathBuf;

pub(crate) enum Cli {
    Help,
    Run {
        force: bool,
        hf: Option<PathBuf>,
        lms: Option<PathBuf>,
    },
}

pub(crate) const HELP: &str = "\
hf2lms - hard-link GGUF models from the Hugging Face cache into LM Studio

Usage:
  hf2lms [OPTIONS]

Options:
  -h, --help           Show this help and exit
  -f, --force          Allow overwriting conflicting files in the models folder
      --hf <PATH>     HF cache 'hub' folder for this run
      --lms <PATH>    LM Studio models folder for this run

On first run both paths are asked interactively and saved to a config file.
Set HF2LMS_CONFIG to use a different config file location.

TUI keys:
  Up/Down or j/k   Move
  Space            Toggle selection
  Enter            Import selected models
  q / Esc          Quit
";

pub(crate) fn parse(args: impl Iterator<Item = String>) -> Result<Cli> {
    let mut force = false;
    let mut hf = None;
    let mut lms = None;
    let mut args = args;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(Cli::Help),
            "-f" | "--force" => force = true,
            "--hf" | "--lms" => {
                let value = args
                    .next()
                    .with_context(|| format!("missing value for {arg}"))?;
                if arg == "--hf" {
                    hf = Some(PathBuf::from(value));
                } else {
                    lms = Some(PathBuf::from(value));
                }
            }
            other => bail!("unknown argument '{other}' (try --help)"),
        }
    }

    Ok(Cli::Run { force, hf, lms })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> impl Iterator<Item = String> {
        values.iter().map(|s| s.to_string())
    }

    #[test]
    fn help_short_circuits() {
        assert!(matches!(parse(args(&["--force", "-h"])), Ok(Cli::Help)));
    }

    #[test]
    fn parses_flags_and_paths() {
        let args = args(&["--hf", "G:\\hf", "-f", "--lms", "D:\\lms"]);
        let Cli::Run { force, hf, lms } = parse(args).unwrap() else {
            panic!("expected Run");
        };
        assert!(force);
        assert_eq!(hf.unwrap(), PathBuf::from("G:\\hf"));
        assert_eq!(lms.unwrap(), PathBuf::from("D:\\lms"));
    }

    #[test]
    fn rejects_bad_arguments() {
        assert!(parse(args(&["--forse"])).is_err());
        assert!(parse(args(&["--hf"])).is_err());
    }
}
