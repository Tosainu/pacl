use std::path::{Path, PathBuf};
use std::process::Command;

use regex::Regex;

use crate::error::{Error, ErrorKind, Result};

#[derive(Debug, PartialEq)]
enum Args {
    Help, // -h, --help
    Clone {
        base_dir: Option<String>, // -b, --base-dir
        prefer_ssh: bool,         // -s, --ssh
        url: String,              // <url>
        extra_args: Vec<String>,  // -- [extra git args] ...
    },
}

pub fn run() -> Result<()> {
    let arg = parse_command_line(std::env::args().skip(1))?;

    match arg {
        Args::Help => {
            print_usage();
            Ok(())
        }
        Args::Clone {
            mut url,
            base_dir,
            prefer_ssh,
            extra_args,
        } => {
            let base_dir = if let Some(d) = base_dir.or_else(base_dir_from_env) {
                PathBuf::from(d)
            } else {
                default_base_dir()?
            };

            if maybe_github_repository(&url) {
                url = if prefer_ssh {
                    format!("{}:{}", "git@github.com", url)
                } else {
                    format!("{}/{}", "https://github.com", url)
                };
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
    println!("    -s, --ssh             prefer SSH to clone GitHub repository");
}

fn parse_command_line(mut args: impl Iterator<Item = String>) -> Result<Args> {
    let mut base_dir = None;
    let mut prefer_ssh = false;
    let mut url = None;
    let mut extra_args = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(Args::Help),

            "-b" | "--base-dir" => {
                if let Some(d) = args.next() {
                    base_dir = Some(d);
                } else {
                    return Err(Error::new(ErrorKind::InvalidArg(Some(arg))));
                }
            }

            "-s" | "--ssh" => prefer_ssh = true,

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
            prefer_ssh,
            url,
            extra_args: extra_args.unwrap_or_default(),
        })
    } else {
        Err(Error::new(ErrorKind::MissingRequiredArg(
            "<url>".to_owned(),
        )))
    }
}

#[test]
fn test_parse_command_line() -> Result<()> {
    assert!(parse_command_line(std::iter::empty()).is_err());

    assert_eq!(parse_command_line(["-h".into()].into_iter())?, Args::Help);
    assert_eq!(
        parse_command_line(["--help".into()].into_iter())?,
        Args::Help
    );
    assert_eq!(
        parse_command_line(["nyan".into(), "-h".into(), "myon".into()].into_iter())?,
        Args::Help
    );

    assert_eq!(
        parse_command_line(["octocat/Spoon-Knife".into()].into_iter())?,
        Args::Clone {
            base_dir: None,
            prefer_ssh: false,
            url: "octocat/Spoon-Knife".into(),
            extra_args: vec![],
        }
    );

    assert!(parse_command_line(["aaa".into(), "bbb".into()].into_iter()).is_err());

    assert_eq!(
        parse_command_line(["-b".into(), "nyan".into(), "octocat/Spoon-Knife".into()].into_iter())?,
        Args::Clone {
            base_dir: Some("nyan".into()),
            prefer_ssh: false,
            url: "octocat/Spoon-Knife".into(),
            extra_args: vec![],
        }
    );
    assert_eq!(
        parse_command_line(["-b".into(), "nyan".into(), "octocat/Spoon-Knife".into()].into_iter())?,
        Args::Clone {
            base_dir: Some("nyan".into()),
            prefer_ssh: false,
            url: "octocat/Spoon-Knife".into(),
            extra_args: vec![],
        }
    );
    assert_eq!(
        parse_command_line(
            [
                "--base-dir".into(),
                "nyan".into(),
                "octocat/Spoon-Knife".into()
            ]
            .into_iter()
        )?,
        Args::Clone {
            base_dir: Some("nyan".into()),
            prefer_ssh: false,
            url: "octocat/Spoon-Knife".into(),
            extra_args: vec![],
        }
    );

    assert!(parse_command_line(["-b".into()].into_iter()).is_err());

    assert_eq!(
        parse_command_line(["-s".into(), "octocat/Spoon-Knife".into()].into_iter())?,
        Args::Clone {
            base_dir: None,
            prefer_ssh: true,
            url: "octocat/Spoon-Knife".into(),
            extra_args: vec![],
        }
    );
    assert_eq!(
        parse_command_line(["--ssh".into(), "octocat/Spoon-Knife".into()].into_iter())?,
        Args::Clone {
            base_dir: None,
            prefer_ssh: true,
            url: "octocat/Spoon-Knife".into(),
            extra_args: vec![],
        }
    );

    assert_eq!(
        parse_command_line(["octocat/Spoon-Knife".into(), "--".into(),].into_iter())?,
        Args::Clone {
            base_dir: None,
            prefer_ssh: false,
            url: "octocat/Spoon-Knife".into(),
            extra_args: vec![],
        }
    );

    assert_eq!(
        parse_command_line(
            [
                "octocat/Spoon-Knife".into(),
                "--".into(),
                "aaa".into(),
                "bbb".into()
            ]
            .into_iter()
        )?,
        Args::Clone {
            base_dir: None,
            prefer_ssh: false,
            url: "octocat/Spoon-Knife".into(),
            extra_args: vec!["aaa".into(), "bbb".into()],
        }
    );

    Ok(())
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
            return Ok(m.get(1).unwrap().as_str().trim_start_matches("git@").into());
        }
    } else {
        // scp-like syntax
        let re = Regex::new(r"^([^/]+?):(~[^/]+?/)?(\S+)$").unwrap();
        if let Some(m) = re.captures(url) {
            return Ok(format!(
                "{}/{}{}",
                m.get(1).unwrap().as_str().trim_start_matches("git@"),
                m.get(2).map(|m| m.as_str()).unwrap_or(""),
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
        git_url_to_path("ssh://git@host:123/foo/bar/baz.git").ok(),
        Some("host:123/foo/bar/baz".to_owned())
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
        Some("user@host/foo/bar/baz.git".to_owned())
    );
    assert_eq!(
        git_url_to_path("git@host:foo/bar/baz.git").ok(),
        Some("host/foo/bar/baz.git".to_owned())
    );
    assert_eq!(
        git_url_to_path("host:~user/foo/bar/baz.git").ok(),
        Some("host/~user/foo/bar/baz.git".to_owned())
    );
    assert_eq!(
        git_url_to_path("host:foo/bar/baz.git").ok(),
        Some("host/foo/bar/baz.git".to_owned())
    );

    assert_eq!(git_url_to_path("").ok(), None);
    assert_eq!(git_url_to_path("/").ok(), None);
    assert_eq!(git_url_to_path("ssh://").ok(), None);
    assert_eq!(git_url_to_path("file:///path/to/repo.git/").ok(), None);
    assert_eq!(git_url_to_path("/path/to/repo.git/").ok(), None);
    assert_eq!(git_url_to_path(":foo/bar/baz.git").ok(), None);
}
