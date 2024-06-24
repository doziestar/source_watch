use env_logger::Env;

mod logger;

pub fn init() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
}