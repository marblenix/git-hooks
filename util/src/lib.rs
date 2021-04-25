use git2::{Config, ErrorCode, Reference, Repository};
use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, ConfigBuilder, LevelPadding, TerminalMode, TermLogger};

// Initialize logging framework
pub fn log_init() {
    let term_logger = TermLogger::new(
        LevelFilter::Debug,
        ConfigBuilder::new()
            .set_level_padding(LevelPadding::Off)
            .build(),
        TerminalMode::Mixed,
        ColorChoice::Never,
    );
    CombinedLogger::init(vec![term_logger]).unwrap();
}

pub fn get_repository() -> Option<Repository> {
    match Repository::open(".") {
        Ok(r) => Some(r),
        Err(e) => {
            log::error!("Not a git directory: {}", e.message());
            None
        }
    }
}

pub fn get_branch_name(repo: &Repository) -> Option<String> {
    let head: Reference<'_> = match repo.head() {
        Ok(head) => head,
        Err(ref e) if e.code() == ErrorCode::UnbornBranch || e.code() == ErrorCode::NotFound => {
            log::error!("Branch has no commits or doesn't exist: {}", e.message());
            return None;
        }
        Err(e) => {
            log::error!("Invalid branch: {}", e.message());
            return None;
        }
    };

    if head.is_branch() && head.shorthand().is_some() {
        return Some(head.shorthand().unwrap().to_string());
    }

    None
}

pub fn get_config(repo: &Repository) -> Option<Config> {
    match repo.config() {
        Ok(config) => Some(config),
        Err(e) => {
            log::error!("Could not get git config from repo: {}", e.message());
            None
        }
    }
}

pub fn get_config_bool(repo: &Repository, key: &str) -> Option<bool> {
    match get_config(repo) {
        None => None,
        Some(config) => match config.get_bool(key) {
            Ok(bool) => Some(bool),
            Err(e) => {
                log::debug!("Could not get bool value from key {}: {}", key, e.message());
                None
            }
        },
    }
}

pub fn get_config_string(repo: &Repository, key: &str) -> Option<String> {
    match get_config(repo) {
        None => None,
        Some(config) => match config.get_string(key) {
            Ok(val) => Some(val),
            Err(e) => {
                log::debug!("Could not get string value from key {}: {}", key, e.message());
                None
            }
        },
    }
}

pub fn get_multi_config_string(repo: &Repository, key: &str) -> Option<Vec<String>> {
    let mut ret: Vec<String> = Vec::new();
    match get_config_string(repo, key) {
        None => None,
        Some(values) => {
            for val in values.split(",") {
                ret.push(val.parse().unwrap())
            }
            Some(ret)
        }
    }
}
