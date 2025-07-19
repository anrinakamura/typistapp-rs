use anyhow::Result;
use clap::Parser;
use typistapp::{model::Model, view::View};

use typistapp::{FONT_DATA, TYPESET};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(value_parser = clap::value_parser!(u32).range(32..=128))]
    length: u32,

    #[arg(short, long)]
    image: String,
}

fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();
    run(&args)
}

fn run(args: &Args) -> Result<()> {
    let mut chars = vec![];
    for c in TYPESET.chars() {
        if c != '\n' {
            chars.push(c);
        }
    }
    log::debug!("Typeset: {chars:?}");

    let image = image::open(&args.image)?;
    log::debug!("Image loaded: {}", args.image);

    let mut m = Model::new(args.length, &image, &chars, FONT_DATA)?;
    log::debug!("Model created: {m:?}");

    let s = m.convert()?;
    for line in &s {
        log::debug!("{line}");
    }

    View::animate(&s)?;
    log::info!("Animation completed successfully!");

    Ok(())
}
