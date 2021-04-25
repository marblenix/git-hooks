use std::env::args;
use std::process::exit;

use git2::Repository;

const DEFAULT_PROTECTED_BRANCHES: [&str; 2] = ["master", "develop"];
const PROTECTED_BRANCHES_SETTING: &'static str = "hooks.pre-push.protectedBranches";
const PRE_PUSH_ENABLED_SETTING: &'static str = "hooks.pre-push.enabled";

fn main() {
    util::log_init();
    args().for_each(|a| log::debug!("ARG: {}", a));
    let repo: Repository = match util::get_repository() {
        None => exit(1),
        Some(r) => r
    };

    if repo.is_bare() {
        log::error!("Cannot check a bare repository");
        exit(1)
    }

    let enabled = util::get_config_bool(&repo, PRE_PUSH_ENABLED_SETTING).unwrap_or(true);
    if !enabled {
        log::warn!("Disabled! Skipping pre-push hook...");
        exit(0);
    }

    let mut protected_branches: Vec<String> =
        util::get_multi_config_string(&repo, PROTECTED_BRANCHES_SETTING).unwrap_or(Vec::new());

    if protected_branches.len() == 0 {
        for branch in DEFAULT_PROTECTED_BRANCHES.to_vec() {
            protected_branches.push(branch.parse().unwrap())
        }
    }

    let branch = match util::get_branch_name(&repo) {
        None => {
            log::error!("Invalid branch or no branch name found");
            exit(1);
        }
        Some(branch_name) => branch_name
    };

    if protected_branches.contains(&branch) {
        log::error!(
            "branch \"{}\" is a protected branch, cancelling push",
            branch
        );
        exit(1)
    }

    exit(1)
}
