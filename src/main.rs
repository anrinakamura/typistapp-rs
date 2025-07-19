use std::io::{BufRead, BufReader};

use anyhow::Result;
use clap::Parser;
use typistapp::{model::Model, view::View};

const FONT: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/", "NotoSansJP-Regular.otf"));

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(value_parser = clap::value_parser!(u32).range(32..=128))]
    length: u32,

    #[arg(short, long, default_value = "resources/monalisa.jpg")]
    image_path: String,
}

fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();
    run(&args)
}

fn run(args: &Args) -> Result<()> {
    let reader = BufReader::new(std::fs::File::open("resources/typeset.txt")?);
    let mut chars = vec![];
    for line in reader.lines() {
        chars.extend(line?.chars());
    }
    log::debug!("Typeset: {chars:?}");

    let image = image::open(&args.image_path)?;
    log::debug!("Image loaded: {}", args.image_path);

    let mut m = Model::new(args.length, &image, &chars, FONT)?;
    log::debug!("Model created: {m:?}");

    let s = m.convert()?;
    for line in &s {
        log::debug!("{line}");
    }

    View::animate(&s)?;
    log::info!("Animation completed successfully!");

    Ok(())
}
