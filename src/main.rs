use std::{
    fs,
    io::{self, Read},
    path::PathBuf,
};

use anyhow::{Context, Result, bail};
use clap::Parser;
use glob::glob;
use maudfmt::{FormatOptions, try_fmt_file};

#[derive(Parser)]
#[command(version, about, long_about = None, arg_required_else_help=true)]
struct Cli {
    /// A space separated list of file, directory or glob
    #[arg(value_name = "FILE", required_unless_present = "stdin")]
    files: Option<Vec<String>>,

    /// Format stdin and write to stdout
    #[arg(short, long, default_value = "false")]
    stdin: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let format_options = FormatOptions::default();

    if cli.stdin {
        let buf = {
            let mut buf = String::new();
            let mut stdin = io::stdin();
            stdin
                .read_to_string(&mut buf)
                .context("Failed to read from stdin")?;
            buf
        };

        let formatted_buf = try_fmt_file(&buf, &format_options).unwrap_or(buf);
        print!("{formatted_buf}");
    } else {
        match cli.files {
            None => bail!("No files provided while not using stdin mode"),
            Some(files) => {
                for file in get_file_paths(files)? {
                    let source = std::fs::read_to_string(&file)?;
                    if let Ok(formatted) = try_fmt_file(&source, &format_options) {
                        if source != formatted {
                            fs::write(file, &formatted)?;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn get_file_paths(input_patterns: Vec<String>) -> Result<Vec<PathBuf>> {
    let mut paths: Vec<PathBuf> = Vec::new();
    for pattern in input_patterns.into_iter().map(as_glob_pattern) {
        for path in glob(&pattern).context(format!("Failed to read glob pattern: {pattern}"))? {
            match path {
                Ok(value) => paths.push(value),
                Err(err) => return Err(err).context("glob error"),
            }
        }
    }
    Ok(paths)
}

fn as_glob_pattern(pattern: String) -> String {
    let is_dir = fs::metadata(&pattern)
        .map(|meta| meta.is_dir())
        .unwrap_or(false);
    if is_dir {
        return format!("{}/**/*.rs", &pattern.trim_end_matches('/'));
    }
    pattern
}
