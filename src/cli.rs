use std::path::{Path, PathBuf};
use std::process::Command;

use regex::Regex;

use crate::error::{Error, ErrorKind, Result};

#[derive(Debug)]
enum Args {
    Help, // -h, --help
    Clone {
        base_dir: Option<String>, // -b, --base-dir
        url: String,              // <url>
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
            mut url,
            base_dir,
            extra_args,
        } => {
            let base_dir = if let Some(d) = base_dir.or_else(base_dir_from_env) {
                PathBuf::from(d)
            } else {
                default_base_dir()?
            };

            if maybe_github_repository(&url) {
                url = format!("{}/{}", "https://github.com", url)
            }

            do_clone(&url, base_dir, &extra_args)
        }
    }
}

fn print_usage() {
    println!("usage:");
    println!("    pacl [options]... <repository url> [-- [extra args passed to git]...]");
    println!();
    println!("options:");
    println!("    -h, --help            display this messages and exit");
    println!("    -b, --base-dir <dir>  base directory to clone");
}

fn parse_command_line() -> Result<Args> {
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
            url,
            extra_args: extra_args.unwrap_or_default(),
        })
    } else {
        Err(Error::new(ErrorKind::MissingRequiredArg(
            "<url>".to_owned(),
        )))
    }
}

fn base_dir_from_env() -> Option<String> {
    std::env::var("PACL_BASE_DIR").ok()
}

fn default_base_dir() -> Result<PathBuf> {
    Ok(home::home_dir()
        .ok_or_else(|| Error::new(ErrorKind::HomeDirectoryNotDetected))?
        .join(".pacl"))
}

fn do_clone<P, S>(url: &str, base_dir: P, extra_args: &[S]) -> Result<()>
where
    P: AsRef<Path>,
    S: AsRef<std::ffi::OsStr>,
{
    let path = git_url_to_path(url)?;
    let path = base_dir.as_ref().join(path);

    let status = Command::new("git")
        .arg("clone")
        .arg(url)
        .arg(path)
        .args(extra_args)
        .spawn()?
        .wait()?;
    match status.code() {
        Some(0) => Ok(()),
        Some(code) => Err(Error::new(ErrorKind::GitReturnedNonZero(code))),
        None => Err(Error::new(ErrorKind::GitTerminated)),
    }
}

fn maybe_github_repository(url: &str) -> bool {
    match url.split_once('/') {
        Some((owner, repository)) => {
            let f1 = |c: char| c.is_ascii_alphanumeric() || c == '-';
            let f2 = |c: char| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_';
            owner.chars().all(f1) && repository.chars().all(f2)
        }
        None => false,
    }
}

#[test]
fn test_maybe_github_repository() {
    assert!(maybe_github_repository("octocat/Spoon-Knife"));
    assert!(maybe_github_repository("octocat/octocat.github.io"));
    assert!(maybe_github_repository("Tosainu/foo_bar"));
    assert!(maybe_github_repository("Tosainu-/foo_bar"));

    assert!(!maybe_github_repository(""));
    assert!(!maybe_github_repository("myon.info"));
    assert!(!maybe_github_repository("myon.info/foo_bar"));
    assert!(!maybe_github_repository("Tosainu=/foo_bar"));
    assert!(!maybe_github_repository("Tosainu_/foo_bar"));
}

fn git_url_to_path(url: &str) -> Result<String> {
    if url.contains("://") {
        let re = Regex::new(r"^\w+?://([^/]\S+?)(?:\.git)?$").unwrap();
        if let Some(m) = re.captures(url) {
            return Ok(m.get(1).unwrap().as_str().into());
        }
    } else {
        // scp-like syntax
        let re = Regex::new(r"^([^/]+?):(~[^/]+?/)?(\S+)$").unwrap();
        if let Some(m) = re.captures(url) {
            return Ok(format!(
                "{}/{}{}",
                m.get(1).unwrap().as_str(),
                m.get(2).map(|m| m.as_str()).unwrap_or("~/"),
                m.get(3).unwrap().as_str()
            ));
        }
    }

    Err(Error::new(ErrorKind::InvalidArg(None)))
}

#[test]
fn test_git_url_to_path() {
    assert_eq!(
        git_url_to_path("https://github.com/octocat/Spoon-Knife").ok(),
        Some("github.com/octocat/Spoon-Knife".to_owned())
    );
    assert_eq!(
        git_url_to_path("https://github.com/octocat/Spoon-Knife.git").ok(),
        Some("github.com/octocat/Spoon-Knife".to_owned())
    );
    assert_eq!(
        git_url_to_path("ssh://user@host:123/foo/bar/baz.git").ok(),
        Some("user@host:123/foo/bar/baz".to_owned())
    );
    assert_eq!(
        git_url_to_path("ssh://user@host/foo/bar/baz.git").ok(),
        Some("user@host/foo/bar/baz".to_owned())
    );
    assert_eq!(
        git_url_to_path("ssh://user@host:123/~user/foo/bar/baz.git").ok(),
        Some("user@host:123/~user/foo/bar/baz".to_owned())
    );
    assert_eq!(
        git_url_to_path("ssh://user@host/~user/foo/bar/baz.git").ok(),
        Some("user@host/~user/foo/bar/baz".to_owned())
    );

    assert_eq!(
        git_url_to_path("user@host:~user/foo/bar/baz.git").ok(),
        Some("user@host/~user/foo/bar/baz.git".to_owned())
    );
    assert_eq!(
        git_url_to_path("user@host:foo/bar/baz.git").ok(),
        Some("user@host/~/foo/bar/baz.git".to_owned())
    );
    assert_eq!(
        git_url_to_path("host:~user/foo/bar/baz.git").ok(),
        Some("host/~user/foo/bar/baz.git".to_owned())
    );
    assert_eq!(
        git_url_to_path("host:foo/bar/baz.git").ok(),
        Some("host/~/foo/bar/baz.git".to_owned())
    );

    assert_eq!(git_url_to_path("").ok(), None);
    assert_eq!(git_url_to_path("/").ok(), None);
    assert_eq!(git_url_to_path("ssh://").ok(), None);
    assert_eq!(git_url_to_path("file:///path/to/repo.git/").ok(), None);
    assert_eq!(git_url_to_path("/path/to/repo.git/").ok(), None);
    assert_eq!(git_url_to_path(":foo/bar/baz.git").ok(), None);
}
