use std::env::{args, Args};
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::exit;

use git2::Repository;
use util::ExitCodes;

const SEPARATOR_SETTING: &'static str = "hooks.prepare-commit-msg.branchSeparator";
const PREPARE_COMMIT_MSG_ENABLED_SETTING: &'static str = "hooks.prepare-commit-msg.enabled";

fn main() {
    util::log_init();
    args().for_each(|a| log::debug!("ARG: {}", a));
    let mut args: Args = args();
    let _binary = args.next();
    let commit_msg_file = args.next();
    let commit_source = args.next();

    let repo: Repository = util::get_repository();

    let enabled = util::get_config_bool(&repo, PREPARE_COMMIT_MSG_ENABLED_SETTING).unwrap_or(true);
    if !enabled {
        log::warn!("{}", ExitCodes::Disabled.message());
        exit(ExitCodes::Disabled.value());
    }

    if commit_source.is_none() && commit_msg_file.is_some() {
        let working_directory = match repo.workdir() {
            None => {
                log::error!("{}", ExitCodes::NoWorkingDirectory.message());
                exit(ExitCodes::NoWorkingDirectory.value())
            }
            Some(working_directory) => working_directory.to_str().unwrap()
        };

        let path: PathBuf = [working_directory, commit_msg_file.unwrap().as_ref()].iter().collect();
        let full_path = path.into_os_string();
        log::debug!("Commit msg file: {:?}", full_path);
        let branch_name = util::get_branch_name(&repo);

        let dynamic_message = format!(
            "{} {} ",
            branch_name,
            util::get_config_string(&repo, SEPARATOR_SETTING).unwrap_or("|".to_string()));

        match prepend_file(dynamic_message.as_str(), full_path.to_str().unwrap()) {
            Ok(_) => {}
            Err(e) => {
                log::trace!("{}", e);
                log::error!("{}", ExitCodes::FailedToWriteCommitMsg.message());
                exit(ExitCodes::FailedToWriteCommitMsg.value());
            }
        }
    }

    log::debug!("{}", ExitCodes::OK.message());
    exit(ExitCodes::OK.value())
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
