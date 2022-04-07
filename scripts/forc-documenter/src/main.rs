use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use std::fs::{create_dir_all, read_to_string, File, OpenOptions};
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use std::process;
use std::str;

pub mod constants;

#[derive(Parser)]
#[clap(name = "forc-documenter", about = "Forc Documenter")]
struct Cli {
    /// the command to run
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    WriteDocs(WriteDocsCommand),
}

#[derive(Debug, Parser)]
struct WriteDocsCommand {
    #[clap(long)]
    pub dry_run: bool,
}

#[derive(PartialEq)]
pub enum LineKind {
    SubHeader,
    Usage,
    Arg,
    Option,
    Text,
}

fn get_sway_path() -> PathBuf {
    let curr_dir = std::env::current_dir().unwrap();
    let sway_dir = curr_dir
        .parent()
        .unwrap()
        .parent()
        .expect("Unable to navigate to project root");
    sway_dir.to_path_buf()
}

fn create_forc_commands_docs_dir(path: &PathBuf) -> Result<()> {
    if !path.is_dir() {
        create_dir_all(&path)?;
    }

    Ok(())
}

fn get_example_for_command(command: &str) -> &str {
    match command {
        "init" => constants::FORC_INIT_EXAMPLE,
        "build" => constants::FORC_BUILD_EXAMPLE,
        "test" => constants::FORC_TEST_EXAMPLE,
        "deploy" => constants::FORC_DEPLOY_EXAMPLE,
        "parse-bytecode" => constants::FORC_PARSE_BYTECODE_EXAMPLE,
        _ => "",
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::WriteDocs(_command) => {
            let WriteDocsCommand { dry_run } = _command;

            let forc_commands_docs_path = get_sway_path().join("docs/src/forc/commands");
            let index_file_path = forc_commands_docs_path.join("index.md");

            if !dry_run {
                create_forc_commands_docs_dir(&forc_commands_docs_path)
                    .expect("Failed to prepare forc commands docs directory");
            }
            let mut index_file = OpenOptions::new()
                .read(true)
                .create(!dry_run)
                .write(!dry_run)
                .open(index_file_path)
                .expect("Problem reading, opening or creating forc/commands/index.md");

            let output = process::Command::new("forc")
                .arg("--help")
                .output()
                .expect("Failed to run help command");

            let s = String::from_utf8_lossy(&output.stdout);
            let lines = s.lines();

            let mut subcommand_is_parsed = false;
            let mut possible_commands = vec![];

            for line in lines {
                if subcommand_is_parsed {
                    let (command, _) = line.trim().split_once(' ').unwrap_or(("", ""));
                    possible_commands.push(command);
                }
                if line == "SUBCOMMANDS:" {
                    subcommand_is_parsed = true;
                }
            }

            let mut index_contents = String::new();
            index_contents.push_str(constants::INDEX_HEADER);

            for command in possible_commands.iter() {
                let mut result = match generate_doc_output(command) {
                    Ok(output) => output,
                    Err(_) => continue,
                };

                let example = get_example_for_command(command);
                if !example.is_empty() {
                    result.push_str(constants::EXAMPLES_HEADER);
                    result.push_str(example);
                }

                let document_name = format_command_doc_name(command);
                let index_entry_name = format_index_entry_name(command);
                let index_entry_string =
                    format_index_entry_string(&document_name, &index_entry_name);

                let forc_command_file_path = forc_commands_docs_path.join(document_name);
                index_contents.push_str(&index_entry_string);

                if dry_run {
                    let existing_contents = read_to_string(&forc_command_file_path);
                    match existing_contents {
                        Ok(existing_contents) => {
                            if existing_contents == result {
                                println!("forc {}: documentation ok.", &command);
                            } else {
                                println!("Documentation inconsistent for {} - please run `cargo run write-docs` within scripts/forc-documenter.", &command);
                                return Err(anyhow!("Documentation inconsistent for {} - please run `cargo run write-docs` within scripts/forc-documenter.", &command));
                            }
                        }
                        Err(_) => {
                            eprintln!("Error: documentation for forc {} does not exist - please run `cargo run write-docs` within scripts/forc-documenter.", &command);
                        }
                    }
                } else {
                    println!("Generating docs for command: forc {}...", &command);
                    let mut command_file = File::create(&forc_command_file_path)
                        .expect("Failed to create documentation");
                    command_file
                        .write_all(result.as_bytes())
                        .expect("Failed to write to file");
                }
            }

            if dry_run {
                let mut existing_index_contents = String::new();
                index_file.read_to_string(&mut existing_index_contents)?;

                if existing_index_contents.is_empty() {
                    eprintln!("Error: failed to read an existing index.md for the commands section - please run `cargo run write-docs` within /scripts/forc-documenter");
                } else {
                    if index_contents == existing_index_contents {
                        println!("index.md ok.");
                    } else {
                        return Err(anyhow!("index.md inconsistent - please run `cargo run write-docs` within scripts/forc-documenter."));
                    }
                }
            } else {
                index_file
                    .write_all(constants::INDEX_HEADER.as_bytes())
                    .expect("Failed to write to forc/commands/index.md");

                index_file
                    .write_all(index_contents.as_bytes())
                    .expect("Failed to write to forc/commands/index.md");
            }

            println!("Done.");
        }
    }
    Ok(())
}

fn format_command_doc_name(command: &str) -> String {
    "forc_".to_owned() + command + ".md"
}

fn format_index_entry_name(command: &str) -> String {
    "forc ".to_owned() + command
}
fn format_index_entry_string(document_name: &str, index_entry_name: &str) -> String {
    "- [".to_owned() + index_entry_name + "](./" + document_name + ")\n"
}

fn generate_doc_output(subcommand: &str) -> Result<String> {
    let mut result = String::new();

    let output = process::Command::new("forc")
        .args([subcommand, "--help"])
        .output()
        .expect("forc --help failed to run");

    if !output.status.success() {
        return Err(anyhow!("Failed to run forc {} --help", subcommand));
    }

    let s = String::from_utf8_lossy(&output.stdout);

    for (index, line) in s.lines().enumerate() {
        let mut formatted_line = String::new();
        let line = line.trim();

        if index == 0 {
            formatted_line.push_str(&format_header_line(line));
        } else if index == 1 {
            formatted_line.push_str(line);
        } else {
            formatted_line.push_str(&format_line(line))
        }

        result.push_str(&formatted_line);

        if !formatted_line.ends_with('\n') {
            result.push('\n');
        }
    }
    Ok(result)
}

fn format_line(line: &str) -> String {
    match get_line_kind(line) {
        LineKind::SubHeader => format_subheader_line(line),
        LineKind::Usage => format_usage_line(line),
        LineKind::Option => format_option_line(line),
        LineKind::Arg => format_arg_line(line),
        LineKind::Text => line.to_string(),
    }
}

fn get_line_kind(line: &str) -> LineKind {
    if constants::SUBHEADERS.contains(&line) {
        LineKind::SubHeader
    } else if is_args_line(line) {
        LineKind::Arg
    } else if is_options_line(line) {
        LineKind::Option
    } else {
        LineKind::Text
    }
}

fn is_args_line(line: &str) -> bool {
    line.trim().starts_with('<')
}

fn is_options_line(line: &str) -> bool {
    line.trim().starts_with('-') && line.trim().chars().nth(1).unwrap() != ' '
}

fn format_header_line(header_line: &str) -> String {
    "\n# ".to_owned() + header_line + "\n"
}

fn format_subheader_line(subheader_line: &str) -> String {
    "\n## ".to_owned() + subheader_line + "\n"
}

fn format_usage_line(usage_line: &str) -> String {
    usage_line.to_string()
}

fn format_arg_line(arg_line: &str) -> String {
    let mut formatted_arg_line = String::new();

    for c in arg_line.chars() {
        if c == '>' {
            formatted_arg_line.push('_');
            formatted_arg_line.push(c);
        } else if c == '<' {
            formatted_arg_line.push(c);
            formatted_arg_line.push('_');
        } else {
            formatted_arg_line.push(c);
        }
    }
    if !formatted_arg_line.trim().ends_with('>') {
        let last_closing_bracket_idx = formatted_arg_line.rfind('>').unwrap();
        formatted_arg_line.replace_range(
            last_closing_bracket_idx + 1..last_closing_bracket_idx + 2,
            "\n\n",
        );
    }
    "\n".to_owned() + &formatted_arg_line
}

fn format_option_line(option_line: &str) -> String {
    let mut tokens_iter = option_line.trim().split(' ');

    let mut result = String::new();
    let mut rest_of_line = String::new();

    while let Some(token) = tokens_iter.next() {
        if is_option(token) {
            result.push_str(&format_option(token));
        } else if is_arg(token) {
            result.push_str(&format_arg(token));
        } else {
            rest_of_line.push_str(token);
            rest_of_line.push(' ');
            rest_of_line = tokens_iter
                .fold(rest_of_line, |mut a, b| {
                    a.reserve(b.len() + 1);
                    a.push_str(b);
                    a.push(' ');
                    a
                })
                .trim()
                .to_owned();
            break;
        }
    }
    result.push_str("\n\n");
    result.push_str(&rest_of_line);
    result.push('\n');

    "\n".to_owned() + &result
}

fn is_option(token: &str) -> bool {
    token.starts_with('-')
}

fn is_arg(token: &str) -> bool {
    token.starts_with('<')
}

fn format_option(option: &str) -> String {
    match option.ends_with(',') {
        true => {
            let mut s = option.to_string();
            s.pop();
            "`".to_owned() + &s + "`, "
        }
        false => "`".to_owned() + option + "` ",
    }
}

fn format_arg(arg: &str) -> String {
    let mut result = String::new();
    let mut inner = arg.to_string();

    inner.pop();
    inner.remove(0);

    result.push('<');
    result.push('_');
    result.push_str(&inner);
    result.push('_');
    result.push('>');

    result
}

#[cfg(test)]
mod tests {
    use crate::is_options_line;

    use super::{format_arg_line, format_header_line, format_option_line, format_subheader_line};

    #[test]
    fn test_format_header_line() {
        let example_header = "forc-fmt";
        let expected_header = "\n# forc-fmt\n";

        assert_eq!(expected_header, format_header_line(example_header));
    }

    #[test]
    fn test_format_subheader_line() {
        let example_subheader = "USAGE:";
        let expected_subheader = "\n## USAGE:\n";

        assert_eq!(expected_subheader, format_subheader_line(example_subheader));
    }

    #[test]
    fn test_format_arg_line() {
        let example_arg_line_1 = "<PROJECT_NAME> Some description";
        let example_arg_line_2 = "<arg1> <arg2> Some description";
        let expected_arg_line_1 = "\n<_PROJECT_NAME_>\n\nSome description";
        let expected_arg_line_2 = "\n<_arg1_> <_arg2_>\n\nSome description";

        assert_eq!(expected_arg_line_1, format_arg_line(example_arg_line_1));
        assert_eq!(expected_arg_line_2, format_arg_line(example_arg_line_2));
    }

    #[test]
    fn test_format_option_line() {
        let example_option_line_1 = "-c, --check    Run in 'check' mode. Exits with 0 if input is formatted correctly. Exits with 1 and prints a diff if formatting is required";
        let example_option_line_2 =
            "-o <JSON_OUTFILE> If set, outputs a json file representing the output json abi";
        let expected_option_line_1= "\n`-c`, `--check` \n\nRun in 'check' mode. Exits with 0 if input is formatted correctly. Exits with 1 and prints a diff if formatting is required\n";
        let expected_option_line_2 = "\n`-o` <_JSON_OUTFILE_>\n\nIf set, outputs a json file representing the output json abi\n";

        assert_eq!(
            expected_option_line_1,
            format_option_line(example_option_line_1)
        );
        assert_eq!(
            expected_option_line_2,
            format_option_line(example_option_line_2)
        );
    }

    #[test]
    fn test_is_options_line() {
        let example_option_line_1= "    -s, --silent             Silent mode. Don't output any warnings or errors to the command line";
        let example_option_line_2 = "    -o <JSON_OUTFILE>        If set, outputs a json file representing the output json abi";
        let example_option_line_3 = " - counter";

        assert!(is_options_line(example_option_line_1));
        assert!(is_options_line(example_option_line_2));
        assert!(!is_options_line(example_option_line_3));
    }
}
