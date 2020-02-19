extern crate num_cpus;
extern crate threadpool;
extern crate walkdir;

use std::env;
use std::io::{stderr, Error, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{channel, Receiver, Sender};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use threadpool::ThreadPool;
use walkdir::WalkDir;

fn main() {
    if env::args().len() <= 1 {
        eprintln!("mgit: Please provide some arguments to pass on to git.");
        return;
    }

    // Allow customizing pool size
    let pool_size: usize = match env::var("MGIT_POOLSIZE") {
        Ok(val) => val.parse().unwrap_or(4),
        Err(_) => num_cpus::get(),
    };

    let (tx, rx) = channel();
    fork(pool_size, &tx);
    drop(tx);
    join(rx).unwrap()
}

fn fork(pool_size: usize, tx: &Sender<(PathBuf, Result<Child, Error>)>) {
    let pool = ThreadPool::new(pool_size);
    for entry in WalkDir::new(env::current_dir().unwrap())
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir() && e.file_name().eq(".git"))
    {
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

            tx.send((path, result)).expect("Could not send data!");
        });
    }
}

fn join(rx: Receiver<(PathBuf, Result<Child, Error>)>) -> Result<(), Error> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let mut success: u8 = 0;
    let mut failed: u8 = 0;
    let mut errors: u8 = 0;
    for (path, result) in rx.iter() {
        match result {
            Ok(child) => {
                let output = child.wait_with_output()?;
                let title_color = match output.status.code() {
                    Some(0) => {
                        success += 1;
                        Some(Color::Green)
                    },
                    Some(_) => {
                        failed += 1;
                        Some(Color::Yellow)
                    },
                    None => {
                        errors += 1;
                        Some(Color::Red)
                    },
                };
                stdout.set_color(ColorSpec::new().set_fg(title_color).set_bold(true))?;
                writeln!(&mut stdout, "{}", path.display())?;
                stdout.reset()?;
                if !&output.stdout.is_empty() {
                    writeln!(&mut stdout, "{}", String::from_utf8_lossy(&output.stdout))?;
                }
                if !&output.stderr.is_empty() {
                    writeln!(&mut stderr(), "{}", String::from_utf8_lossy(&output.stderr))?;
                }
            }
            Err(err) => {
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))?;
                writeln!(&mut stderr(), "{}", path.display())?;
                stdout.reset()?;
                writeln!(&mut stderr(), "{}", err)?;
                errors += 1;
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
