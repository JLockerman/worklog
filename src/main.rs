use std::{
    convert::TryInto,
    ffi::{OsStr, OsString},
    fs::{self, File},
    fmt::Write as _,
    io::SeekFrom,
    io::{Read, Seek, Write},
    path::{Path, PathBuf},
    process::exit,
    process::Command,
};

use clap::Clap;

use chrono::prelude::*;
use fs::OpenOptions;

fn main() {
    use Opts::*;
    let opts = Opts::parse();
    match opts {
        Today { log_directory } => {
            let worklog = ensure_todays_entry(&log_directory);
            let status = Command::new("open")
                .arg(worklog.path.as_os_str())
                .status()
                .expect("could not run open command");
            if !status.success() {
                exit(1)
            }
        }
        EndTask { log_directory } => {
            let worklog = ensure_todays_entry(&log_directory);
            end_last_task(worklog);
        }
        StartTask { log_directory } => {
            let worklog = ensure_todays_entry(&log_directory);
            let worklog = end_last_task(worklog);
            start_new_task(worklog);
        }
    };
    exit(0)
}

#[derive(Clap)]
#[clap(version = "1.0")]
enum Opts {
    Today { log_directory: OsString },
    StartTask { log_directory: OsString },
    EndTask { log_directory: OsString },
}

struct LogFile {
    path: PathBuf,
    file: File,
    contents: String,
    todays_header: String,
}

fn ensure_todays_entry(log_directory: &OsStr) -> LogFile {
    let today = Local::now();
    let mut week_start = today.date();
    while week_start.weekday() != Weekday::Sun {
        week_start = week_start.pred()
    }

    let log_directory = Path::new(log_directory);
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

    let mut log_file = open_for_read(&log_filename);

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
        let mut log_file = open_for_append(&log_filename);
        write!(log_file, "{}\n\n", todays_header).unwrap_or_else(|e| {
            eprintln!(
                "ERROR: failed to write today's header to '{}': {}",
                log_filename.display(),
                e,
            );
            exit(1)
        });

        log_file = open_for_read(&log_filename);
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
    }

    LogFile {
        path: log_filename,
        file: log_file,
        contents: log_contents,
        todays_header,
    }
}

fn start_new_task(worklog: LogFile) {
    let mut output = String::with_capacity(20);
    if !worklog.contents.ends_with("\n\n") {
        if !worklog.contents.ends_with("\n") {
            output.push('\n')
        }
        output.push('\n')
    }

    let now = Local::now();
    let _ = write!(&mut output, "{} - __:__", now.format("%R"));

    let mut log_file = open_for_append(&worklog.path);
    writeln!(log_file, "{}", output).expect("cannot write new task");
}

fn end_last_task(mut worklog: LogFile) -> LogFile {
    let header_offset = worklog.contents.rfind(&worklog.todays_header).unwrap();
    let mut contents = &worklog.contents[..];
    let last_unfinished_task = loop {
        // TODO is there a way to check if the placeholder isn't in the last task?
        let last_unfinished_task = contents.rfind("__:__");
        let last_unfinished_task = match last_unfinished_task {
            None => return worklog,
            Some(last_unfinished_task) => last_unfinished_task,
        };

        if last_unfinished_task <= header_offset {
            return worklog;
        }

        if worklog.contents.as_bytes()[last_unfinished_task - 9] != b'\n' {
            contents = &contents[..last_unfinished_task];
            continue;
        }

        break last_unfinished_task;
    };

    worklog
        .file
        .seek(SeekFrom::Start(last_unfinished_task.try_into().unwrap()))
        .expect("cannot reach last unfinished task");

    let now = Local::now();
    write!(&mut worklog.file, "{}", now.format("%R")).expect("unable to write end time");
    // TODO reread or replace the contents?
    return worklog;
}

fn open_for_read(log_filename: &Path) -> File {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(log_filename)
        .unwrap_or_else(|e| {
            eprintln!(
                "ERROR: unable to open file '{}': {}",
                log_filename.display(),
                e,
            );
            exit(1)
        })
}

fn open_for_append(log_filename: &Path) -> File {
    OpenOptions::new()
        .append(true)
        .open(log_filename)
        .unwrap_or_else(|e| {
            eprintln!(
                "ERROR: unable to open file for append '{}': {}",
                log_filename.display(),
                e,
            );
            exit(1)
        })
}
