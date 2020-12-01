use std::{
    convert::TryInto, ffi::OsString, fs, io::Read, io::Write, path::Path, process::exit,
    process::Command,
};

use clap::Clap;

use chrono::prelude::*;
use fs::OpenOptions;

fn main() {
    use Opts::*;
    let opts = Opts::parse();
    match opts {
        Today(today) => today.run(),
        StartTask(start) => start.run(),
        EndTask(end) => end.run(),
    }
}

#[derive(Clap)]
#[clap(version = "1.0")]
enum Opts {
    Today(Today),
    StartTask(StartTask),
    EndTask(EndTask),
}

#[derive(Clap)]
struct Today {
    log_directory: OsString,
}

impl Today {
    fn run(self) {
        let today = Local::now();
        let mut week_start = today.date();
        while week_start.weekday() != Weekday::Sun {
            week_start = week_start.pred()
        }

        let log_directory = Path::new(&self.log_directory);
        if !log_directory.exists() {
            eprintln!(
                "ERROR: log directory '{}' does not exist",
                log_directory.display()
            );
            exit(1)
        }

        let month_directory = log_directory.join(week_start.format("%Y-%m (%B %Y)").to_string());
        fs::create_dir_all(&month_directory).unwrap_or_else(|e| {
            eprintln!(
                "ERROR: creating month directory '{}': {}",
                month_directory.display(),
                e,
            );
            exit(1)
        });

        let log_filename = month_directory
            .join(week_start.format("%Y-%m-%d").to_string())
            .with_extension("worklog");
        let mut log_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&log_filename)
            .unwrap_or_else(|e| {
                eprintln!(
                    "ERROR: unable to open file '{}': {}",
                    log_filename.display(),
                    e,
                );
                exit(1)
            });

        let log_len = log_file.metadata().map(|m| m.len()).unwrap_or(0);
        let mut log_contents = String::with_capacity(log_len.try_into().unwrap());
        log_file
            .read_to_string(&mut log_contents)
            .unwrap_or_else(|e| {
                eprintln!(
                    "ERROR: unable to read file '{}': {}",
                    log_filename.display(),
                    e,
                );
                exit(1)
            });
        let todays_header = format!(
            "{:10}{}\n====================",
            today.format("%A"),
            today.format("%F")
        );
        if !log_contents.contains(&todays_header) {
            let mut log_file = OpenOptions::new()
                .append(true)
                .open(&log_filename)
                .unwrap_or_else(|e| {
                    eprintln!(
                        "ERROR: unable to open file for append '{}': {}",
                        log_filename.display(),
                        e,
                    );
                    exit(1)
                });
            write!(log_file, "{}\n\n", todays_header).unwrap_or_else(|e| {
                eprintln!(
                    "ERROR: failed to write today's header to '{}': {}",
                    log_filename.display(),
                    e,
                );
                exit(1)
            })
        }

        let status = Command::new("open")
            .arg(log_filename.as_os_str())
            .status()
            .expect("could not run open command");
        if !status.success() {
            exit(1)
        }

        exit(0)
    }
}

#[derive(Clap)]
struct StartTask {}

impl StartTask {
    fn run(self) {}
}

#[derive(Clap)]
struct EndTask {}

impl EndTask {
    fn run(self) {}
}
