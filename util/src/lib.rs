use git2::{Config, ErrorCode, Reference, Repository};
use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, ConfigBuilder, LevelPadding, TerminalMode, TermLogger};

pub enum ExitCode {
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

impl ExitCode {
    pub fn value(&self) -> i32 {
        match &self {
            ExitCode::OK => 0,
            ExitCode::Disabled => 0,
            ExitCode::FailedToOpenRepository => 1,
            ExitCode::RepositoryIsBare => 2,
            ExitCode::NoWorkingDirectory => 3,
            ExitCode::InvalidBranch => 4,
            ExitCode::EmptyBranch => 5,
            ExitCode::UnknownBranch => 6,
            ExitCode::BadBranchName => 7,
            ExitCode::ProtectedBranch => 8,
            ExitCode::FailedToWriteCommitMsg => 9,
        }
    }
    pub fn message(&self) -> &str {
        match &self {
            ExitCode::OK => "Success!",
            ExitCode::Disabled => "Disabled! Skipping git hook",
            ExitCode::FailedToOpenRepository => "Not a git directory",
            ExitCode::RepositoryIsBare => "Repository is empty",
            ExitCode::NoWorkingDirectory => "Repository has no working directory",
            ExitCode::InvalidBranch => "Invalid branch",
            ExitCode::EmptyBranch => "Branch has no commits",
            ExitCode::UnknownBranch => "HEAD is not a branch",
            ExitCode::BadBranchName => "Branch name is invalid UTF-8",
            ExitCode::ProtectedBranch => "HEAD refers to a protected branch",
            ExitCode::FailedToWriteCommitMsg => "Failed to write commit message to file",
        }
    }
}

pub fn fatal(code: ExitCode) -> ! {
    log::error!("{}", code.message());
    std::process::exit(code.value())
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
            fatal(ExitCode::FailedToOpenRepository);
        }
    }
}

pub fn get_branch_name(repo: &Repository) -> String {
    let head: Reference<'_> = match repo.head() {
        Ok(head) => head,
        Err(ref e) if e.code() == ErrorCode::UnbornBranch => {
            log::trace!("{}", e);
            fatal(ExitCode::EmptyBranch);
        }
        Err(e) => {
            log::trace!("{}", e);
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
