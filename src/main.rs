use std::{
    io::{stdin, BufRead, BufReader, Write},
    path::PathBuf,
    process::{Command, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
    thread,
    time::Duration,
};

use dashmap::DashSet;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

fn main() {
    let kernels: Vec<_> = stdin()
        .lines()
        .map(|x| x.unwrap())
        .filter(|x| x.ends_with("vmlinuz"))
        .collect();

    let stderr_messages = &DashSet::new();
    let suppressed = &AtomicUsize::new(0);

    let m = &MultiProgress::new();
    let style = &ProgressStyle::with_template("{spinner:.green} {wide_msg}")
        .unwrap()
        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏");

    thread::scope(|scope| {
        for kernel in &kernels {
            let pb = m.add(ProgressBar::new(0));
            pb.set_style(style.clone());
            pb.enable_steady_tick(Duration::from_millis(150));

            let mut child = Command::new("/usr/share/libalpm/scripts/mkinitcpio")
                .arg("install")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();

            let mut child_stdin = child.stdin.take().unwrap();

            let path = PathBuf::from("/").join(kernel).canonicalize().unwrap();

            child_stdin
                .write_all(path.to_str().unwrap().as_bytes())
                .unwrap();
            child_stdin.write_all("\n".as_bytes()).unwrap();
            child_stdin.flush().unwrap();
            drop(child_stdin);

            let stdout = child.stdout.take().unwrap();
            let stdout = BufReader::new(stdout);

            scope.spawn(move || {
                for line in stdout.lines() {
                    pb.set_message(format!("[{}] {}", kernel, line.unwrap().trim()));
                }

                pb.finish_with_message(format!("[{kernel}] Done!"));
            });

            let stderr = child.stderr.take().unwrap();
            let stderr = BufReader::new(stderr);

            scope.spawn(move || {
                for line in stderr.lines() {
                    let line = line.unwrap();
                    if stderr_messages.insert(line.clone()) {
                        m.suspend(|| println!("[{}] {}", kernel, line.trim()));
                    } else {
                        suppressed.fetch_add(1, Ordering::SeqCst);
                    }
                }
            });
        }
    });

    println!("Generation complete!");
    if !stderr_messages.is_empty() {
        let suppressed = suppressed.load(Ordering::SeqCst);
        println!(
            "{} warnings ({} suppressed)",
            suppressed + stderr_messages.len(),
            suppressed
        );
    }
}
