use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use indicatif_log_bridge::LogWrapper;
use log::{info, LevelFilter};
use std::time::{Duration, SystemTime};

static mut MULTI: Option<MultiProgress> = None;


pub fn initialize_logging(log_level: LevelFilter) {
    let logger = env_logger::builder()
        .filter_level(log_level)
        .parse_default_env() // Allow overriding log level through RUST_LOG env var
        .build();

    let multi = MultiProgress::new();

    let wrapper = LogWrapper::new(multi.clone(), logger);
    wrapper.try_init().unwrap();

    unsafe {
        MULTI = Some(multi);
    }
}


pub fn run_with_spinner<'a, F, Out>(
    target: &'a str, task_desc: &'a str, function: F,
) -> Out where
    F: FnOnce() -> Out,
{
    let start_time = SystemTime::now();

    let pb = ProgressBar::new_spinner()
        .with_message(format!("{}...", task_desc))
        .with_style(ProgressStyle::with_template("{spinner:.white} [{elapsed:.green}] {msg}").unwrap());
    pb.enable_steady_tick(Duration::from_millis(100));

    // Set up connection with log library so that progress bars don't jump around
    unsafe {
        MULTI.clone().unwrap().add(pb.clone());
    };

    let out = function();

    pb.finish_and_clear();
    unsafe { MULTI.clone().unwrap().remove(&pb); }
    let elapsed = indicatif::HumanDuration(start_time.elapsed().unwrap());
    info!(target: target, "{} finished (took {})", task_desc, elapsed);

    out
}

pub fn run_with_pb<'a, F, Out>(
    target: &'a str, task_desc: &'a str, total: u64, print_message: bool, function: F,
) -> Out where
    F: FnOnce(ProgressBar) -> Out,
{
    let start_time = SystemTime::now();
    
    let pb = ProgressBar::new(total)
        .with_message(format!("{}...", task_desc))
        .with_style(
            ProgressStyle::with_template("[{elapsed:.green}] {msg} [{wide_bar:.cyan/blue}] {human_pos}/{human_len} [{eta}]")
                .unwrap().progress_chars("=> ")
        );
    pb.enable_steady_tick(Duration::from_secs(1));
    
    unsafe {
        MULTI.clone().unwrap().add(pb.clone());
    }
    
    let out = function(pb.clone());
    
    pb.finish_and_clear();
    unsafe { MULTI.clone().unwrap().remove(&pb); }
    if print_message {
        let elapsed = indicatif::HumanDuration(start_time.elapsed().unwrap());
        info!(target: target, "{} finished (took {})", task_desc, elapsed);
    }
    
    out
}