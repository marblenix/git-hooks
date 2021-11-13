use git2::{Config, ErrorCode, Reference, Repository};
use logging::{fatal, ExitCode};

pub fn get_repository() -> Repository {
    match Repository::open(".") {
        Ok(r) => r,
        Err(e) => {
            logging::trace_m(e.message());
            fatal(ExitCode::FailedToOpenRepository);
        }
    }
}

pub fn get_branch_name(repo: &Repository) -> String {
    let head: Reference<'_> = match repo.head() {
        Ok(head) => head,
        Err(ref e) if e.code() == ErrorCode::UnbornBranch => {
            logging::trace_m(e.message());
            fatal(ExitCode::EmptyBranch);
        }
        Err(e) => {
            logging::trace_m(e.message());
            fatal(ExitCode::InvalidBranch);
        }
    };

    if head.is_branch() {
        match head.shorthand() {
            Some(branch) => return branch.to_string(),
            None => {
                fatal(ExitCode::BadBranchName);
            }
        }
    }

    fatal(ExitCode::UnknownBranch);
}

pub fn get_config(repo: &Repository) -> Option<Config> {
    match repo.config() {
        Ok(config) => Some(config),
        Err(e) => {
            let msg = format!("Could not get git config from repo: {}", e.message());
            logging::error_m(msg.as_str());
            None
        }
    }
}

pub fn get_config_bool(repo: &Repository, key: &str) -> Option<bool> {
    match get_config(repo) {
        Some(config) => match config.get_bool(key) {
            Ok(bool) => Some(bool),
            Err(e) => {
                let msg = format!("Could not get bool value from key {}: {}", key, e.message());
                logging::debug_m(msg.as_str());
                None
            }
        },
        None => None,
    }
}

pub fn get_config_string(repo: &Repository, key: &str) -> Option<String> {
    match get_config(repo) {
        Some(config) => match config.get_string(key) {
            Ok(val) => Some(val),
            Err(e) => {
                let msg = format!(
                    "Could not get string value from key {}: {}",
                    key,
                    e.message()
                );
                logging::debug_m(msg.as_str());
                None
            }
        },
        None => None,
    }
}

pub fn get_multi_config_string(repo: &Repository, key: &str) -> Option<Vec<String>> {
    let mut ret: Vec<String> = Vec::new();
    match get_config_string(repo, key) {
        Some(values) => {
            for val in values.split(",") {
                ret.push(val.parse().unwrap())
            }
            Some(ret)
        }
        None => None,
    }
}
