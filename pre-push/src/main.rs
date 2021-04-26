use std::env::args;
use std::process::exit;

use git2::Repository;
use util::ExitCodes;

const DEFAULT_PROTECTED_BRANCHES: [&str; 2] = ["master", "develop"];
const PROTECTED_BRANCHES_SETTING: &'static str = "hooks.pre-push.protectedBranches";
const PRE_PUSH_ENABLED_SETTING: &'static str = "hooks.pre-push.enabled";

fn main() {
    util::log_init();
    args().for_each(|a| log::debug!("ARG: {}", a));
    let repo: Repository = util::get_repository();

    if repo.is_bare() {
        log::error!("{}", ExitCodes::RepositoryIsBare.message());
        exit(ExitCodes::RepositoryIsBare.value())
    }

    let enabled = util::get_config_bool(&repo, PRE_PUSH_ENABLED_SETTING).unwrap_or(true);
    if !enabled {
        log::warn!("{}", ExitCodes::Disabled.message());
        exit(ExitCodes::Disabled.value());
    }

    let mut protected_branches: Vec<String> =
        util::get_multi_config_string(&repo, PROTECTED_BRANCHES_SETTING).unwrap_or(Vec::new());

    if protected_branches.len() == 0 {
        for branch in DEFAULT_PROTECTED_BRANCHES.to_vec() {
            protected_branches.push(branch.parse().unwrap())
        }
    }

    let branch_name = util::get_branch_name(&repo);
    if protected_branches.contains(&branch_name) {
        log::error!("{}", ExitCodes::ProtectedBranch.message());
        exit(ExitCodes::ProtectedBranch.value())
    }

    log::debug!("{}", ExitCodes::OK.message());
    exit(ExitCodes::OK.value())
}
