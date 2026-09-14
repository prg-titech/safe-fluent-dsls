use fastrace::collector::{Config, ConsoleReporter};
use sqls::server::Backend;

#[tokio::main]
async fn main() {
    stderrlog::new()
        .modules([module_path!(), "sqls"])
        .verbosity(4)
        .init()
        .unwrap();

    fastrace::set_reporter(ConsoleReporter, Config::default());

    Backend::stdio().await;
}