use std::time::{Duration, SystemTime};

use indicatif::{ProgressBar, ProgressStyle};
use log::info;

pub fn run_with_spinner<F, Out>(
    target: &'static str, task_desc: &'static str, function: F,
) -> Out where F: FnOnce() -> Out {
    let start_time = SystemTime::now();
    let pb = ProgressBar::new_spinner()
        .with_message(format!("{}...", task_desc))
        .with_style(ProgressStyle::with_template("{spinner} [{elapsed}] {msg}").unwrap());
    pb.enable_steady_tick(Duration::from_millis(100));

    let out = function();

    pb.finish_and_clear();
    info!(target: target, "{} finished (took {:?})", task_desc, start_time.elapsed().unwrap());

    out
}