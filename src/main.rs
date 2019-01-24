//! # rscli
//! The rust-based command-line interface.
//!
//! ## Summary
//! This program acts as a command-line interpreter to rust.

// ----- External Crates -----
extern crate clap;
extern crate rustyline;
// ---------------------------

// Import some useful macros from the "clap" crate
use clap::{crate_authors,crate_description,crate_version};

// Import "ReadlineError" from "rustyline"
use rustyline::error::ReadlineError;

// Ability to run commands.
use std::process::Command;
use std::io::{self, Write};
use std::error::Error;
use std::fs::File;
use std::path::Path;

/// Handles a line of text.
fn handle_line(prompt: &str, shell: &mut rustyline::Editor<()>) -> bool {
    let nextline = shell.readline(prompt);
    match nextline {
        Ok(line) => {
            let history = shell.history().iter().map(|s| &**s).collect::<Vec<_>>().join("\n");
            let source_code = &["#[allow(unused)]\nfn main() {\n", history.as_str(), "\n", line.as_str(), "\n}"].concat();
            let file_path = Path::new("/tmp/rscli.rs");
            let mut file = match File::create(&file_path) {
                Err(why) => panic!("couldn't create file: {}", why.description()),
                Ok(file) => file,
            };
            file.write_all(source_code.as_bytes()).expect("couldn't write to file");
            let compile_output = Command::new("rustc")
                .arg("/tmp/rscli.rs")
                .arg("-o")
                .arg("/tmp/rscli.out")
                .output()
                .expect("failed to compile code...");
            io::stdout().write_all(&compile_output.stdout).unwrap();
            io::stderr().write_all(&compile_output.stderr).unwrap();
            if compile_output.status.success() {
                let run_output = Command::new("/tmp/rscli.out").output().expect("Failed to run command");
                io::stdout().write_all(&run_output.stdout).unwrap();
                io::stderr().write_all(&run_output.stderr).unwrap();
                shell.add_history_entry(line.as_ref());
            }
            return true;
        },
        Err(ReadlineError::Interrupted) => {
            return false;
        },
        Err(ReadlineError::Eof) => {
            return false;
        },
        Err(err) => {
            eprintln!("Error: {:?}", err);
            return false;
        }
    }
}

/// The main entry point of the program.
fn main() {
    // Parse command-line arguments
    let args = parse_arguments();

    // Start the interactive shell
    let mut shell = rustyline::Editor::<()>::new();

    // Load a session file if specified
    let session_file = args.value_of("session_file");
    if let Some(f) = session_file {
        if shell.load_history(f).is_err() {
            eprintln!("Unable to load specified session file...");
        }
    }

    // Enter the main process loop
    loop {
        if let Some(f) = session_file {
            if !handle_line(&[f," | > "].concat(), &mut shell) {
                break;
            }
        }
        else {
            if !handle_line("> ", &mut shell) {
                break;
            }
        }
    }

    // Save the history to the specified file
    if let Some(f) = session_file {
        if shell.save_history(f).is_err() {
            eprintln!("Unable to save specified session file...");
        }
    }
}

/// Parses the command-line arguments passed into the program.
fn parse_arguments<'a>() -> clap::ArgMatches<'a> {
    let argument_parser = clap::App::new("rscli")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(clap::Arg::with_name("log_file")
             .short("f")
             .long("log-file")
             .value_name("FILE")
             .help("Specifies the log file to write events to.")
             .default_value("~/.rscli.log")
        )
        .arg(clap::Arg::with_name("log_level")
             .short("l")
             .long("log-level")
             .value_name("LVL")
             .help("Specifies the log level to write at.")
             .possible_values(&["info", "debug"])
             .default_value("info")
        )
        .arg(clap::Arg::with_name("log_mode")
             .short("m")
             .long("log-mode")
             .value_name("MODE")
             .help("Specifies whether to append to or overwrite an existing log file.")
             .possible_values(&["append", "overwrite"])
             .default_value("append")
        )
        .arg(clap::Arg::with_name("session_file")
             .short("s")
             .long("session-file")
             .value_name("FILE")
             .help("Specifies a file to load containing evaluations from a previous session. If no previous file was found, a new one will be created when the shell is exited.")
        );
    return argument_parser.get_matches();
}
