use crate::{ADDON_DIR, ADDON_TRANSPOSE, reset_noctavox};
use anyhow::Result;
use clap::{ArgGroup, Parser};
use std::{path::PathBuf, process::Command};

#[derive(Parser, Debug)]
#[command(
    name = "NoctaVox",
    version,
    about = "A TUI music player for local files"
)]
#[command(group(
      ArgGroup::new("mode")
          .args(["import_playlist", "export_playlist", "list", "reset"]),
  ))]
struct Cli {
    /// Import a playlist from a csv or m3u file
    #[arg(long, short)]
    import_playlist: bool,

    /// Export a playlist to m3u, csv, or json format
    #[arg(long, short)]
    export_playlist: bool,

    /// List playlists in the library
    #[arg(long)]
    list: bool,

    /// Destroy database completely
    #[arg(long)]
    reset: bool,
}

pub fn parse_args() {
    let cli = Cli::parse();

    if cli.import_playlist {
        let _ = run_addon(ADDON_TRANSPOSE, &["--import"]);
    } else if cli.export_playlist {
        let _ = run_addon(ADDON_TRANSPOSE, &["--export"]);
    } else if cli.list {
        let _ = run_addon(ADDON_TRANSPOSE, &["--list"]);
    } else if cli.reset {
        let _ = reset_noctavox();
    };
}

fn addon_path(name: &str) -> PathBuf {
    if let Ok(entries) = std::fs::read_dir(&*ADDON_DIR) {
        let mut matches: Vec<PathBuf> = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_str()
                    .map(|s| s.starts_with(name))
                    .unwrap_or(false)
            })
            .map(|e| e.path())
            .collect();

        match matches.len() {
            1 => return matches.remove(0),
            n if n > 1 => {
                eprintln!(
                    "Multiple matches for addon `{name}` in {}:",
                    ADDON_DIR.display()
                );
                for m in &matches {
                    eprintln!("  {}", m.display());
                }
                eprintln!("Keep only one, or rename the correct one to `{name}`.");
                std::process::exit(1);
            }
            _ => {}
        }
    }

    ADDON_DIR.join(format!("{name}{}", std::env::consts::EXE_SUFFIX))
}

fn run_addon(name: &str, params: &[&str]) -> Result<i32> {
    let bin = addon_path(name);
    if !bin.exists() {
        eprintln!(
            "Addon `{name}` not found.\nExpected addon in: {}/\n\nDownload `{name}` at https://github.com/Jaxx497/NoctaVox-Plugins/releases/latest/\n",
            ADDON_DIR.display()
        );
        std::process::exit(1)
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&bin)?.permissions().mode();
        if mode & 0o111 == 0 {
            eprintln!("addon at {} is not executable (chmod +x it)", bin.display());
            std::process::exit(1);
        }
    }

    let status = Command::new(&bin)
        .args(params)
        .status()
        .unwrap_or_else(|e| {
            eprintln!("Error: {e}");
            std::process::exit(1)
        });

    Ok(status.code().unwrap_or(1))
}
