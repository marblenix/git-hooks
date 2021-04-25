use std::env::{args, Args};
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::exit;

use git2::Repository;

const SEPARATOR_SETTING: &'static str = "hooks.prepare-commit-msg.branchSeparator";
const PREPARE_COMMIT_MSG_ENABLED_SETTING: &'static str = "hooks.prepare-commit-msg.enabled";

fn main() {
    util::log_init();
    let mut args: Args = args();
    let binary = args.next();
    let commit_msg_file = args.next();
    let commit_source = args.next();
    log::debug!("binary: {:?}", binary);

    let repo: Repository = match util::get_repository() {
        None => exit(1),
        Some(r) => r
    };

    let enabled = util::get_config_bool(&repo, PREPARE_COMMIT_MSG_ENABLED_SETTING).unwrap_or(true);
    if !enabled {
        log::warn!("Disabled! Skipping prepare-commit-msg hook...");
        exit(0);
    }

    if commit_source.is_none() && commit_msg_file.is_some() {
        // this is a new commit, go ahead and prepare a template

        let path: PathBuf = [
            repo.workdir().expect("Repo has no working directory").to_str().unwrap(),
            commit_msg_file.unwrap().as_ref()
        ].iter().collect();
        let full_path = path.into_os_string();
        log::debug!("PATH: {:?}", full_path);

        match util::get_branch_name(&repo) {
            None => {
                log::error!("Failed to get branch name for repository, refusing to create a commit message template");
                exit(0);
            }
            Some(branch_name) => {
                let dynamic_message = format!(
                    "{} {} ",
                    branch_name,
                    util::get_config_string(&repo, SEPARATOR_SETTING).unwrap_or("|".to_string()));

                match prepend_file(dynamic_message.as_str(), full_path.to_str().unwrap()) {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("Failed to write to file: {}", e);
                        exit(1);
                    }
                }
            }
        }
    }

    exit(0)
}

fn prepend_file(data: &str, file_path: &str) -> std::io::Result<()> {
    let mut src = File::open(&file_path)?;
    let mut contents = String::new();

    src.read_to_string(&mut contents)?;
    contents = format!("{}\n{}", data, contents);

    let mut dest = File::create(&file_path)?;
    dest.write_all(contents.as_ref())?;
    Ok(())
}
