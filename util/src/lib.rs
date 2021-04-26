use git2::{Config, ErrorCode, Reference, Repository};
use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, ConfigBuilder, LevelPadding, TerminalMode, TermLogger};
use std::process::exit;

pub enum ExitCodes {
    OK,
    Disabled,
    FailedToOpenRepository,
    RepositoryIsBare,
    NoWorkingDirectory,
    InvalidBranch,
    EmptyBranch,
    UnknownBranch,
    BadBranchName,
    ProtectedBranch,
    FailedToWriteCommitMsg,
}

impl ExitCodes {
    pub fn value(&self) -> i32 {
        match &self {
            ExitCodes::OK => 0,
            ExitCodes::Disabled => 0,
            ExitCodes::FailedToOpenRepository => 1,
            ExitCodes::RepositoryIsBare => 2,
            ExitCodes::NoWorkingDirectory => 3,
            ExitCodes::InvalidBranch => 4,
            ExitCodes::EmptyBranch => 5,
            ExitCodes::UnknownBranch => 6,
            ExitCodes::BadBranchName => 7,
            ExitCodes::ProtectedBranch => 8,
            ExitCodes::FailedToWriteCommitMsg => 9,
        }
    }
    pub fn message(&self) -> &str {
        match &self {
            ExitCodes::OK => "Success!",
            ExitCodes::Disabled => "Disabled! Skipping git hook",
            ExitCodes::FailedToOpenRepository => "Not a git directory",
            ExitCodes::RepositoryIsBare => "Repository is empty",
            ExitCodes::NoWorkingDirectory => "Repository has no working directory",
            ExitCodes::InvalidBranch => "Invalid branch",
            ExitCodes::EmptyBranch => "Branch has no commits",
            ExitCodes::UnknownBranch => "HEAD is not a branch",
            ExitCodes::BadBranchName => "Branch name is invalid UTF-8",
            ExitCodes::ProtectedBranch => "HEAD refers to a protected branch",
            ExitCodes::FailedToWriteCommitMsg => "Failed to write commit message to file",
        }
    }
}

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

pub fn get_repository() -> Repository {
    match Repository::open(".") {
        Ok(r) => r,
        Err(e) => {
            log::trace!("{}", e);
            log::error!("{}", ExitCodes::FailedToOpenRepository.message());
            exit(ExitCodes::FailedToOpenRepository.value());
        }
    }
}

pub fn get_branch_name(repo: &Repository) -> String {
    let head: Reference<'_> = match repo.head() {
        Ok(head) => head,
        Err(ref e) if e.code() == ErrorCode::UnbornBranch => {
            log::trace!("{}", e);
            log::error!("{}", ExitCodes::EmptyBranch.message());
            exit(ExitCodes::EmptyBranch.value());
        }
        Err(e) => {
            log::trace!("{}", e);
            log::error!("{}", ExitCodes::InvalidBranch.message());
            exit(ExitCodes::InvalidBranch.value());
        }
    };

    if head.is_branch() {
        match head.shorthand() {
            Some(branch) => return branch.to_string(),
            None => {
                log::error!("{}", ExitCodes::BadBranchName.message());
                exit(ExitCodes::BadBranchName.value());
            }
        }
    }

    log::error!("{}", ExitCodes::UnknownBranch.message());
    exit(ExitCodes::UnknownBranch.value());
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
        Some(config) => match config.get_bool(key) {
            Ok(bool) => Some(bool),
            Err(e) => {
                log::debug!("Could not get bool value from key {}: {}", key, e.message());
                None
            }
        },
        None => None
    }
}

pub fn get_config_string(repo: &Repository, key: &str) -> Option<String> {
    match get_config(repo) {
        Some(config) => match config.get_string(key) {
            Ok(val) => Some(val),
            Err(e) => {
                log::debug!("Could not get string value from key {}: {}", key, e.message());
                None
            }
        },
        None => None
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
        },
        None => None
    }
}
