use log::{info, warn, error};

pub fn init() {
    env_logger::init();
    info!("Logger initialized");
}

pub fn log_info(message: &str) {
    info!("{}", message);
}

pub fn log_warn(message: &str) {
    warn!("{}", message);
}

pub fn log_error(message: &str) {
    error!("{}", message);
}
