use log::{LevelFilter, info};
use rstest::Context;
use serde_json::{Map, Value};
use simplelog::{ConfigBuilder, WriteLogger};
use std::fs::File;
use std::path::PathBuf;
use std::str::FromStr;

pub fn configure_logger(ctx: Context) {
    let raw_output: String = String::from_utf8(
        escargot::Cargo::new()
            .arg("metadata")
            .into_command()
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    let output = serde_json::from_str::<Map<String, Value>>(&raw_output).unwrap();
    let module_as_path =
        PathBuf::from_str(&ctx.module.split("::").collect::<Vec<_>>().join("/")).unwrap();
    let target_path = PathBuf::from_str(output.get("build_directory").unwrap().as_str().unwrap())
        .unwrap()
        .join("test")
        .join(module_as_path)
        .join(format!("{}.log", ctx.name));
    std::fs::create_dir_all(target_path.parent().unwrap()).unwrap();

    let logger_config = ConfigBuilder::new().build();
    WriteLogger::init(
        LevelFilter::Trace,
        logger_config,
        File::create(target_path).unwrap(),
    )
    .unwrap();
    info!("Set up logger!");
}
