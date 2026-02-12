//! Progress bar utilities

use indicatif::{ProgressBar, ProgressStyle};

/// Create a standard progress bar
#[allow(dead_code)]
pub fn create_progress_bar(total: u64, message: &str) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
            .expect("invalid progress bar template")
            .progress_chars("##-"),
    );
    pb.set_message(message.to_string());
    pb
}

/// Create a spinner for indeterminate progress
#[allow(dead_code)]
pub fn create_spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .expect("invalid spinner template"),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    pb
}
