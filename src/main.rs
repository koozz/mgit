extern crate num_cpus;
extern crate threadpool;
extern crate walkdir;

use std::env;
use std::fmt::Write as FmtWrite;
use std::io::{stderr, Error, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc::{channel, Receiver, Sender};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use threadpool::ThreadPool;
use walkdir::WalkDir;

enum GitStatus {
    SUCCESS,
    FAILURE,
    ERROR,
}

fn main() {
    if env::args().len() <= 1 {
        eprintln!("mgit: Please provide some arguments to pass on to git.");
        return;
    }

    // Allow customizing pool size
    let pool_size: usize = match env::var("MGIT_PARALLEL") {
        Ok(val) => val.parse().unwrap_or(4),
        Err(_) => num_cpus::get(),
    };

    let (tx, rx) = channel();
    fork(pool_size, &tx).unwrap();
    drop(tx);
    join(rx).unwrap()
}

fn fork(pool_size: usize, tx: &Sender<(PathBuf, GitStatus, String)>) -> Result<(), Error> {
    let mut count = 0;
    let pool = ThreadPool::new(pool_size);
    for entry in WalkDir::new(env::current_dir()?)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir() && e.file_name().eq(".git"))
    {
        count += 1;
        let path = entry.path().parent().unwrap().to_owned();
        let tx = tx.clone();
        pool.execute(move || {
            let args: Vec<_> = env::args().skip(1).collect();
            let result = Command::new("git")
                .current_dir(path.clone())
                .args(args)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn();

            let mut output = String::new();
            let status: GitStatus;
            match result {
                Ok(child) => {
                    let child_output = child.wait_with_output().unwrap();

                    status = match child_output.status.code() {
                        Some(0) => GitStatus::SUCCESS,
                        Some(_) => GitStatus::FAILURE,
                        None => GitStatus::ERROR,
                    };
                    if !&child_output.stdout.is_empty() {
                        writeln!(
                            &mut output,
                            "{}",
                            String::from_utf8_lossy(&child_output.stdout)
                        )
                        .unwrap();
                    }
                    if !&child_output.stderr.is_empty() {
                        writeln!(
                            &mut output,
                            "{}",
                            String::from_utf8_lossy(&child_output.stderr)
                        )
                        .unwrap();
                    }
                }
                Err(err) => {
                    status = GitStatus::ERROR;
                    writeln!(&mut output, "{}", err).unwrap();
                }
            }

            tx.send((path, status, output))
                .expect("Could not send data!");
        });
    }

    let git_arguments: Vec<String> = env::args().skip(1).collect();
    println!(
        "Running 'git {}' on {} projects.",
        git_arguments.join(" "),
        count
    );
    Ok(())
}

fn join(rx: Receiver<(PathBuf, GitStatus, String)>) -> Result<(), Error> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let mut success = 0;
    let mut failed = 0;
    let mut errors = 0;
    for (path, status, output) in rx.iter() {
        let title_color = match status {
            GitStatus::SUCCESS => {
                success += 1;
                Some(Color::Green)
            }
            GitStatus::FAILURE => {
                failed += 1;
                Some(Color::Yellow)
            }
            GitStatus::ERROR => {
                errors += 1;
                Some(Color::Red)
            }
        };

        stdout.set_color(ColorSpec::new().set_fg(title_color).set_bold(true))?;
        writeln!(&mut stdout, "{}", path.display())?;
        stdout.reset()?;

        match status {
            GitStatus::SUCCESS => {
                write!(&mut stdout, "{}", output)?;
            }
            GitStatus::FAILURE | GitStatus::ERROR => {
                write!(&mut stderr(), "{}", output)?;
            }
        };
    }

    // Status line:
    writeln!(&mut stdout)?;
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)).set_bold(true))?;
    write!(&mut stdout, "Success: ")?;
    stdout.reset()?;
    write!(&mut stdout, "{}", success)?;

    if failed > 0 {
        write!(&mut stdout, ", ")?;
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)).set_bold(true))?;
        write!(&mut stdout, "Warnings: ")?;
        stdout.reset()?;
        write!(&mut stdout, "{}", failed)?;
    }

    if errors > 0 {
        write!(&mut stdout, ", ")?;
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))?;
        write!(&mut stdout, "Errors: ")?;
        stdout.reset()?;
        write!(&mut stdout, "{}", errors)?;
    }
    writeln!(&mut stdout)?;
    Ok(())
}
