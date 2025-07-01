use std::io::{BufRead, BufReader};

use anyhow::Result;
use clap::Parser;
use typistapp::model::Model;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    length: u32,

    #[arg(short, long, default_value = "resources/monalisa.jpg")]
    image_path: String,

    #[arg(short, long, default_value = "resources/NotoSansJP-Regular.otf")]
    font_path: String,
}

fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();
    run(&args)
}

fn run(args: &Args) -> Result<()> {
    if args.length < 32 || 128 < args.length {
        log::error!("Length should be between 32 and 128. Please try again.");
        return Err(anyhow::anyhow!("Invalid length"));
    }

    let reader = BufReader::new(std::fs::File::open("resources/typeset.txt")?);
    let mut chars = vec![];
    for line in reader.lines() {
        chars.extend(line?.chars());
    }
    log::debug!("chars: {:?}", chars);

    let image = image::open(&args.image_path)?;
    log::debug!("Image loaded: {}", args.image_path);

    let font_data = std::fs::read(&args.font_path)?;
    let mut m = Model::from_vec(font_data)?;
    log::debug!("Model created: {:?}", m);

    m.run(args.length, &chars, &image)?;

    Ok(())
}
