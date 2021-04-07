use std::path::{Path, PathBuf};

use crate::error::{Error, ErrorKind, Result};
use crate::url::normalize_repo_url;

#[derive(Debug)]
pub enum Args {
    Help, // -h, --help
    Clone {
        base_dir: Option<String>, // -b, --base-dir
        url: url::Url,            // <url>
        extra_args: Vec<String>,  // -- [extra git args] ...
    },
}

pub fn run() -> Result<()> {
    let arg = parse_command_line()?;

    match arg {
        Args::Help => {
            print_usage();
            Ok(())
        }
        Args::Clone {
            url,
            base_dir,
            extra_args,
        } => {
            let base_dir = if let Some(d) = base_dir {
                PathBuf::from(d)
            } else {
                default_base_dir()?
            };
            do_clone(url, base_dir.as_path(), extra_args)
        }
    }
}

pub fn print_usage() {
    println!("usage:");
    println!("    pacl [options]... <repository url> [-- [extra args passed to git]...]");
    println!();
    println!("options:");
    println!("    -h, --help            display this messages and exit");
    println!("    -b, --base-dir <dir>  base directory to clone");
}

pub fn parse_command_line() -> Result<Args> {
    let mut args = std::env::args();
    args.next();

    let mut base_dir = None;
    let mut url = None;
    let mut extra_args = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(Args::Help),

            "-b" | "--base-dir" => base_dir = Some(args.next().unwrap()),

            "--" => {
                extra_args = Some(args.collect());
                break;
            }

            _ => {
                if url.is_none() {
                    url = Some(arg)
                } else {
                    return Err(Error::new(ErrorKind::InvalidArg(Some(arg))));
                }
            }
        }
    }

    if let Some(url) = url {
        Ok(Args::Clone {
            base_dir,
            url: normalize_repo_url(url)?,
            extra_args: extra_args.unwrap_or_default(),
        })
    } else {
        Err(Error::new(ErrorKind::MissingRequiredArg(
            "<url>".to_owned(),
        )))
    }
}

fn default_base_dir() -> Result<PathBuf> {
    Ok(dirs::home_dir()
        .ok_or_else(|| Error::new(ErrorKind::HomeDirectoryNotDetected))?
        .join(".pacl"))
}

fn do_clone(url: url::Url, base_dir: &Path, extra_args: Vec<String>) -> Result<()> {
    use std::process::Command;

    let host = match (url.host_str(), url.port()) {
        (Some(host), Some(port)) => format!("{}:{}", host, port),
        (Some(host), None) => String::from(host),
        _ => unreachable!(),
    };

    let path = url.path();
    let path = path.strip_suffix(".git").unwrap_or(path);

    let dir = base_dir.join(host).join(&path[1..]);

    let mut cmd = Command::new("git");
    cmd.arg("clone")
        .arg(url.to_string())
        .arg(dir)
        .args(&extra_args);

    println!("$ {:?}", cmd);

    match cmd.spawn()?.wait()?.code() {
        Some(0) => Ok(()),
        Some(code) => Err(Error::new(ErrorKind::GitReturnedNonZero(code))),
        None => Err(Error::new(ErrorKind::GitTerminated)),
    }
}
