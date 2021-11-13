use logging::{fatal, ExitCode};
use regex::Regex;
use std::env::{args, Args};
use std::fmt::Formatter;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::exit;

const PREPARE_COMMIT_MSG_ENABLED_DEFAULT: &'static bool = &true;
const PREPARE_COMMIT_MSG_ENABLED_SETTING: &'static str = "hooks.prepare-commit-msg.enabled";
const SEPARATOR_DEFAULT: &'static str = "|";
const SEPARATOR_SETTING: &'static str = "hooks.prepare-commit-msg.branchSeparator";

const JIRA_REGEX: &'static str = r"((([A-Z]{1,10})-?)[A-Z]+-\d+)";

#[derive(Debug, Eq, PartialEq)]
pub struct Meta {
    branch_type: String,
    ticket: String,
    description: String,
}

impl std::fmt::Display for Meta {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let repo = git_bindings::get_repository();
        let separator = git_bindings::get_config_string(&repo, SEPARATOR_SETTING)
            .unwrap_or(SEPARATOR_DEFAULT.to_string());

        write!(f, "{}", self.to_msg(separator))
    }
}

impl Meta {
    fn branch_type(branch: &String) -> String {
        if branch.contains('/') {
            let split: Vec<&str> = branch.split('/').collect();
            return split.get(0).unwrap().to_string();
        }

        return match branch.as_str() {
            s @ "master"
            | s @ "main"
            | s @ "develop"
            | s @ "feature"
            | s @ "release"
            | s @ "hotfix" => s,
            _ => "",
        }
        .to_string();
    }

    fn ticket(branch: &String) -> String {
        let regex = &Regex::new(JIRA_REGEX).unwrap();
        if !(regex.is_match(branch.as_str())) {
            return String::new();
        }

        let captures = regex.captures(branch.as_str());
        if captures.is_none() {
            return String::new();
        }

        let m = captures.unwrap().get(0);
        if m.is_some() {
            m.unwrap().as_str().to_string()
        } else {
            String::new()
        }
    }

    fn description(branch: &String, branch_type: &String, ticket: &String) -> String {
        if branch.contains('/') {
            return if ticket == &String::new() {
                let split: Vec<&str> = branch.split('/').collect();
                split.get(1).unwrap_or(&"").to_string()
            } else {
                let split: Vec<&str> = branch.split(ticket.as_str()).collect();
                let val: String = split.get(1).unwrap_or(&"").to_string();
                val.trim()
                    .trim_matches(|c: char| c.is_ascii_punctuation())
                    .to_string()
            };
        }

        if ticket == &String::new() {
            return if branch_type == &String::new() {
                branch.clone()
            } else {
                String::new()
            };
        }

        String::new()
    }

    pub fn new(branch_string: String) -> Self {
        let branch = branch_string.trim().to_string();

        let branch_type: String = Self::branch_type(&branch);
        let ticket: String = Self::ticket(&branch);
        let description: String = Self::description(&branch, &branch_type, &ticket);

        Self {
            branch_type,
            ticket,
            description,
        }
    }

    pub fn to_msg(&self, separator: String) -> String {
        if self.ticket == String::new() && self.description == String::new() {
            return String::new();
        }

        if self.ticket == String::new() {
            return format!(
                "{description} {separator} ",
                separator = separator,
                description = self.description
            );
        }

        if self.description == String::new() {
            return format!(
                "{ticket} {separator} ",
                ticket = self.ticket,
                separator = separator
            );
        }

        format!(
            "{ticket} {separator} {description}",
            ticket = self.ticket,
            separator = separator,
            description = self.description
        )
    }
}

fn main() {
    logging::log_init();
    logging::log_args(args());

    let mut args: Args = args();
    let _binary = args.next();
    let commit_msg_file = args.next();
    let commit_source = args.next();
    let repo = git_bindings::get_repository();

    let enabled = git_bindings::get_config_bool(&repo, PREPARE_COMMIT_MSG_ENABLED_SETTING)
        .unwrap_or(*PREPARE_COMMIT_MSG_ENABLED_DEFAULT);
    if !enabled {
        logging::warn(ExitCode::Disabled);
        exit(ExitCode::Disabled.value());
    }

    if commit_source.is_none() && commit_msg_file.is_some() {
        let working_directory = match repo.workdir() {
            None => fatal(ExitCode::NoWorkingDirectory),
            Some(working_directory) => working_directory.to_str().unwrap(),
        };

        let path: PathBuf = [working_directory, commit_msg_file.unwrap().as_ref()]
            .iter()
            .collect();
        let full_path = path.into_os_string();
        let msg = format!("Commit msg file: {:?}", full_path);
        logging::debug_m(msg.as_str());
        let branch_name = git_bindings::get_branch_name(&repo);

        let separator = git_bindings::get_config_string(&repo, SEPARATOR_SETTING)
            .unwrap_or(SEPARATOR_DEFAULT.to_string());

        let message = Meta::new(branch_name).to_msg(separator);

        match prepend_file(message.as_str(), full_path.to_str().unwrap()) {
            Ok(_) => {}
            Err(e) => {
                logging::trace_m(e.to_string().as_str());
                fatal(ExitCode::FailedToWriteCommitMsg);
            }
        }
    }

    logging::debug(ExitCode::OK);
    exit(ExitCode::OK.value())
}

fn prepend_file(data: &str, file_path: &str) -> std::io::Result<()> {
    let mut src = File::open(&file_path)?;
    let mut contents = String::new();

    src.read_to_string(&mut contents)?;
    contents = format!("{}\n{}", data, contents);

    let mut dest = File::create(&file_path)?;
    dest.write_all(contents.as_ref())?;
    Ok(())
}

#[cfg(test)]
mod meta_tests {
    use crate::Meta;

    #[test]
    fn blank() {
        let expected = Meta {
            branch_type: String::new(),
            ticket: String::new(),
            description: String::new(),
        };

        assert_eq!(expected, Meta::new(String::new()))
    }

    #[test]
    fn just_the_type() {
        let expected = Meta {
            branch_type: String::from("master"),
            ticket: String::new(),
            description: String::new(),
        };

        assert_eq!(expected, Meta::new("master".to_string()))
    }

    #[test]
    fn just_the_ticket() {
        let expected = Meta {
            branch_type: String::new(),
            ticket: String::from("FOO-911"),
            description: String::new(),
        };

        assert_eq!(expected, Meta::new("FOO-911".to_string()))
    }

    #[test]
    fn just_the_description() {
        let expected = Meta {
            branch_type: String::new(),
            ticket: String::new(),
            description: String::from("some-description"),
        };

        assert_eq!(expected, Meta::new("some-description".to_string()))
    }

    #[test]
    fn no_jira_ticket() {
        let expected = Meta {
            branch_type: String::from("feature"),
            ticket: String::new(),
            description: String::from("some-description"),
        };

        assert_eq!(expected, Meta::new("feature/some-description".to_string()))
    }

    #[test]
    fn it_finds_jira_tickets() {
        let expected = Meta {
            branch_type: String::from("feature"),
            ticket: String::from("JIRA-302"),
            description: String::from("some-description"),
        };

        for branch in [
            "feature/JIRA-302-some-description",
            "feature/JIRA-302_some-description",
            "feature/JIRA-302!some-description",
            "feature/JIRA-302~some-description",
        ] {
            let meta = Meta::new(branch.to_string());
            assert_eq!(expected, meta)
        }
    }
}

#[cfg(test)]
mod msg_tests {
    use crate::Meta;

    #[test]
    fn blank() {
        let meta = Meta {
            branch_type: String::new(),
            ticket: String::new(),
            description: String::new(),
        };

        let expected = String::new();
        assert_eq!(expected, meta.to_msg(String::from("|")))
    }

    #[test]
    fn just_the_type() {
        let meta = Meta {
            branch_type: String::from("master"),
            ticket: String::new(),
            description: String::new(),
        };

        let expected = String::new();
        assert_eq!(expected, meta.to_msg(String::from("|")))
    }

    #[test]
    fn just_the_ticket() {
        let meta = Meta {
            branch_type: String::new(),
            ticket: String::from("FOO-911"),
            description: String::new(),
        };

        let expected = String::from("FOO-911 | ");
        assert_eq!(expected, meta.to_msg(String::from("|")))
    }

    #[test]
    fn just_the_description() {
        let meta = Meta {
            branch_type: String::new(),
            ticket: String::new(),
            description: String::from("some-description"),
        };

        let expected = String::from("some-description | ");
        assert_eq!(expected, meta.to_msg(String::from("|")))
    }

    #[test]
    fn no_jira_ticket() {
        let meta = Meta {
            branch_type: String::from("feature"),
            ticket: String::new(),
            description: String::from("some-description"),
        };

        let expected = String::from("some-description | ");
        assert_eq!(expected, meta.to_msg(String::from("|")))
    }

    #[test]
    fn it_finds_jira_tickets() {
        let meta = Meta {
            branch_type: String::from("feature"),
            ticket: String::from("JIRA-302"),
            description: String::from("some-description"),
        };

        let expected = String::from("JIRA-302 | some-description");
        assert_eq!(expected, meta.to_msg(String::from("|")))
    }

    #[test]
    fn it_gives_a_commit_msg() {
        let meta = Meta {
            branch_type: String::from("feature"),
            ticket: String::from("JIRA-302"),
            description: String::from("some-description"),
        };

        let expected = String::from("JIRA-302 | some-description");
        assert_eq!(expected, meta.to_msg(String::from("|")))
    }
}
