use std::{
    env::args,
    fs::read_dir,
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
    let args: Vec<_> = args().collect();
    let kernels: Vec<_> = if args.get(1).map(|x| x.as_str()) == Some("--all") {
        read_dir("/usr/lib/modules")
            .unwrap()
            .map(|x| x.unwrap())
            .filter_map(|x| {
                let path = x.path();
                read_dir(path)
                    .unwrap()
                    .map(|x| x.unwrap())
                    .find(|x| x.file_name() == "vmlinuz")
                    .map(|x| x.path().to_str().unwrap().chars().skip(1).collect())
            })
            .collect::<Vec<_>>()
    } else {
        stdin()
            .lines()
            .map(|x| x.unwrap())
            .filter(|x| x.ends_with("vmlinuz"))
            .collect()
    };

    if kernels.is_empty() {
        println!("use `fast-mkinitcpio --all` to regenerate");
        return;
    }

    let stdout_messages = &DashSet::new();
    let stderr_messages = &DashSet::new();
    let suppressed = &AtomicUsize::new(0);

    let m = &MultiProgress::new();
    let style = &ProgressStyle::with_template("{spinner:.green} {wide_msg}")
        .unwrap()
        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏");

    let mut children = vec![];

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
                    let line = line.unwrap();
                    let line = line.trim();
                    pb.set_message(format!("[{}] {}", kernel, line));
                    if line.contains("hook: [") && stdout_messages.insert(line.to_string()) {
                        m.suspend(|| println!("[{}] {}", kernel, line));
                    }
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

            children.push(child);
        }
    });

    for mut child in children {
        let status = child.wait().unwrap();
        assert!(status.success());
    }

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
