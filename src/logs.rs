use std::fs::{File, remove_file};
use std::path::Path;
use log4rs::append::file::FileAppender;
use log4rs::Config;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log::LevelFilter;

pub fn init_log(){

    let path = Path::new("/home/victor/RustroverProjects/chessig2/output.log");
    let logs_file = File::open(path).unwrap_or(File::create(path).unwrap());
    logs_file.set_len(0).ok();
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build(path).unwrap();
    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder()
            .appender("logfile")
            .build(LevelFilter::Info)).unwrap();
    log4rs::init_config(config).unwrap();
}