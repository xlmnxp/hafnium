extern crate simplelog;
use simplelog::*;
use std::fs::File;

pub fn setup() {
    let logger_file = File::create("hafnium.log").unwrap();
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Warn, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
            WriteLogger::new(LevelFilter::Warn, Config::default(), logger_file.try_clone().unwrap()),
            TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
            WriteLogger::new(LevelFilter::Info, Config::default(), logger_file),
        ]
    ).unwrap();
}