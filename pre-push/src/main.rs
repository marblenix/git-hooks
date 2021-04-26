use std::env::args;
use std::process::exit;

use git2::Repository;
use util::ExitCode;

const DEFAULT_PROTECTED_BRANCHES: [&str; 2] = ["master", "develop"];
const PROTECTED_BRANCHES_SETTING: &'static str = "hooks.pre-push.protectedBranches";
const PRE_PUSH_ENABLED_SETTING: &'static str = "hooks.pre-push.enabled";

fn main() {
    util::log_init();
    args().for_each(|a| log::debug!("ARG: {}", a));
    let repo: Repository = util::get_repository();

    if repo.is_bare() {
        util::fatal(ExitCode::RepositoryIsBare)
    }

    let enabled = util::get_config_bool(&repo, PRE_PUSH_ENABLED_SETTING).unwrap_or(true);
    if !enabled {
        log::warn!("{}", ExitCode::Disabled.message());
        exit(ExitCode::Disabled.value());
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
        util::fatal(ExitCode::ProtectedBranch)
    }

    log::debug!("{}", ExitCode::OK.message());
    exit(ExitCode::OK.value())
}
