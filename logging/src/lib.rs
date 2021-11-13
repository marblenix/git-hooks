use log::LevelFilter;
use simplelog::{
    ColorChoice, CombinedLogger, ConfigBuilder, LevelPadding, TermLogger, TerminalMode,
};
use std::env::Args;

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

// Initialize logging framework
pub fn log_init() {
    let term_logger = TermLogger::new(
        log::STATIC_MAX_LEVEL,
        ConfigBuilder::new()
            .set_level_padding(LevelPadding::Off)
            .build(),
        TerminalMode::Mixed,
        ColorChoice::Never,
    );
    CombinedLogger::init(vec![term_logger]).unwrap();
}

pub fn log_args(args: Args) {
    let current_log_level = log::STATIC_MAX_LEVEL as usize;
    if current_log_level < LevelFilter::Debug as usize {
        return;
    }

    let mut i = 0;
    for arg in args {
        let msg = format!("ARG[{}]: {}", i, arg);
        debug_m(msg.as_str());
        i += 1;
    }
}

pub fn fatal(code: ExitCode) -> ! {
    log::error!("{}", code.message());
    std::process::exit(code.value())
}

pub fn error(code: ExitCode) {
    log::error!("{}", code.message())
}

pub fn warn(code: ExitCode) {
    log::warn!("{}", code.message())
}

pub fn info(code: ExitCode) {
    log::info!("{}", code.message())
}

pub fn debug(code: ExitCode) {
    log::debug!("{}", code.message())
}

pub fn trace(code: ExitCode) {
    log::trace!("{}", code.message())
}

pub fn error_m(message: &str) {
    log::error!("{}", message)
}

pub fn warn_m(message: &str) {
    log::warn!("{}", message)
}

pub fn info_m(message: &str) {
    log::info!("{}", message)
}

pub fn debug_m(message: &str) {
    log::debug!("{}", message)
}

pub fn trace_m(message: &str) {
    log::trace!("{}", message)
}
