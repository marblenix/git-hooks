use std::env::args;
use std::process::exit;

use git_bindings::{get_branch_name, get_config_bool, get_multi_config_string, get_repository};
use logging::{fatal, ExitCode};

const PRE_PUSH_ENABLED_DEFAULT: &'static bool = &true;
const PRE_PUSH_ENABLED_SETTING: &'static str = "hooks.pre-push.enabled";
const PROTECTED_BRANCHES_DEFAULT: [&str; 2] = ["master", "develop"];
const PROTECTED_BRANCHES_SETTING: &'static str = "hooks.pre-push.protectedBranches";

fn main() {
    logging::log_init();
    args()
        .map(|arg| format!("ARG: {}", arg))
        .for_each(|arg| logging::debug_m(arg.as_str()));
    let repo = get_repository();

    if repo.is_bare() {
        fatal(ExitCode::RepositoryIsBare)
    }

    let enabled =
        get_config_bool(&repo, PRE_PUSH_ENABLED_SETTING).unwrap_or(*PRE_PUSH_ENABLED_DEFAULT);
    if !enabled {
        logging::warn(ExitCode::Disabled);
        exit(ExitCode::Disabled.value());
    }

    let protected_branches: Vec<String> =
        get_multi_config_string(&repo, PROTECTED_BRANCHES_SETTING).unwrap_or(Vec::from(
            PROTECTED_BRANCHES_DEFAULT.map(|b| String::from(b)),
        ));

    let branch_name = get_branch_name(&repo);
    if protected_branches.contains(&branch_name) {
        fatal(ExitCode::ProtectedBranch)
    }

    logging::debug(ExitCode::OK);
    exit(ExitCode::OK.value())
}
