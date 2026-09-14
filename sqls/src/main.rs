use log::LevelFilter;
use sqls::server::Backend;

#[tokio::main]
async fn main() {
    let log_level = if cfg!(debug_assertions) {
        LevelFilter::Trace
    } else {
        LevelFilter::Info
    };
    let log_path = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join(".sqls.log");

    simple_logging::log_to_file(log_path, log_level.into()).unwrap();

    Backend::stdio().await;
}