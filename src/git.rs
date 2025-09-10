use crate::error::{GarnixError, Result};
use git2::Repository;

pub fn get_current_branch() -> Result<String> {
    let repo = Repository::open(".")?;

    let head_ref = repo.head();

    match head_ref {
        Ok(head) => {
            if let Some(name) = head.shorthand() {
                Ok(name.to_string())
            } else if let Some(name) = head.name() {
                if name.starts_with("refs/heads/") {
                    Ok(name.strip_prefix("refs/heads/").unwrap().to_string())
                } else {
                    Err(GarnixError::Git(git2::Error::from_str(
                        "Could not extract branch name from HEAD",
                    )))
                }
            } else {
                Err(GarnixError::Git(git2::Error::from_str(
                    "Could not get branch name from HEAD",
                )))
            }
        }
        Err(_) => match std::fs::read_to_string(".git/HEAD") {
            Ok(content) => {
                let content = content.trim();
                if content.starts_with("ref: refs/heads/") {
                    Ok(content
                        .strip_prefix("ref: refs/heads/")
                        .unwrap()
                        .to_string())
                } else {
                    Err(GarnixError::Git(git2::Error::from_str(
                        "HEAD is not pointing to a branch",
                    )))
                }
            }
            Err(_) => Err(GarnixError::Git(git2::Error::from_str(
                "Could not read .git/HEAD",
            ))),
        },
    }
}

pub fn is_git_repository() -> bool {
    Repository::open(".").is_ok()
}

pub fn get_branch_or_override(override_branch: Option<String>) -> Result<String> {
    match override_branch {
        Some(branch) => Ok(branch),
        None => {
            if !is_git_repository() {
                return Err(GarnixError::NotInGitRepo);
            }
            get_current_branch()
        }
    }
}
