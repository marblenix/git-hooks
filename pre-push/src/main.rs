extern crate log;
extern crate simplelog;

use std::process::exit;

use git2::{ErrorCode, Reference, Repository, Config};
use simplelog::{CombinedLogger, LevelFilter, TermLogger, TerminalMode};

const DEFAULT_PROTECTED_BRANCHES: [&str; 2] = ["master", "develop"];
const PROTECTED_BRANCHES_SETTING: &'static str = "hooks.pre-push.protectedBranches";
const PRE_PUSH_ENABLED_SETTING: &'static str = "hooks.pre-push.enabled";

fn main() {
    let term_logger =
        TermLogger::new(LevelFilter::Debug, Default::default(), TerminalMode::Mixed).unwrap();
    CombinedLogger::init(vec![term_logger]).unwrap();

    let repo: Repository = match Repository::open(".") {
        Ok(r) => r,
        Err(e) => {
            log::error!(
                "Not a git directory: \
                pre-push belongs in the ./.git/hooks directory and should not be called manually\n{}",
                e
            );
            exit(1);
        }
    };

    if repo.is_bare() {
        log::error!("Cannot check a bare repository");
        exit(1)
    }

    let enabled = get_config_bool(&repo, PRE_PUSH_ENABLED_SETTING).unwrap_or(true);
    if !enabled {
        log::warn!("Disabled! Skipping pre-push checks...");
        exit(0);
    }

    let mut protected_branches: Vec<String> =
        get_multi_config_string(&repo, PROTECTED_BRANCHES_SETTING).unwrap_or(Vec::new());

    if protected_branches.len() == 0 {
        for branch in DEFAULT_PROTECTED_BRANCHES.to_vec() {
            protected_branches.push(branch.parse().unwrap())
        }
    }

    let head: Reference = match repo.head() {
        Ok(head) => head,
        Err(ref e) if e.code() == ErrorCode::UnbornBranch || e.code() == ErrorCode::NotFound => {
            log::error!("branch has no commits or doesn't exist: {}", e.message());
            exit(1)
        }
        Err(_) => {
            log::error!("invalid branch");
            exit(1)
        }
    };

    if head.is_branch() {
        let branch: &String = &match head.shorthand() {
            Some(s) => s.to_string(),
            None => {
                log::error!("No branch name?");
                exit(1);
            },
        };

        log::debug!("current branch: {}", branch);
        log::debug!("protected branches: {:?}", protected_branches);

        if protected_branches.contains(branch) {
            log::error!(
                "branch \"{}\" is a protected branch, cancelling push",
                branch
            );
            exit(1)
        }
    }

    exit(0)
}

fn get_config(repo: &Repository) -> Option<Config> {
    match repo.config() {
        Ok(config) => Some(config),
        Err(e) => {
            log::error!("Could not get git config from repo\n{}", e);
            None
        },
    }
}

fn get_config_bool(repo: &Repository, key: &str) -> Option<bool> {
    match get_config(repo) {
        None => None,
        Some(config) => {
            match config.get_bool(key) {
                Ok(bool) => Some(bool),
                Err(e) => {
                    log::info!("Could not get bool value from key {}: {}", key, e);
                    None
                },
            }
        },
    }
}

fn get_config_string(repo: &Repository, key: &str) -> Option<String> {
    match get_config(repo) {
        None => None,
        Some(config) => {
            match config.get_string(key) {
                Ok(val) => Some(val),
                Err(e) => {
                    log::info!("Could not get string value from key {}: {}", key, e);
                    None
                },
            }
        },
    }
}

fn get_multi_config_string(repo: &Repository, key: &str) -> Option<Vec<String>> {
    let mut ret: Vec<String> = Vec::new();
    match get_config_string(repo, key) {
        None => None,
        Some(vals) => {
            for val in vals.split(",") {
                ret.push(val.parse().unwrap())
            }
            Some(ret)
        }
    }
}
