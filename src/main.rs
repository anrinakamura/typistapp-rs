use std::io::{BufRead, BufReader};

use anyhow::Result;
use typistapp::{model::Model, view::View};

fn main() -> Result<()> {
    env_logger::init();

    run()
}

fn run() -> Result<()> {
    let reader = BufReader::new(std::fs::File::open("resources/typeset.txt")?);
    let mut chars = vec![];
    for line in reader.lines() {
        chars.extend(line?.chars());
    }
    log::debug!("chars: {:?}", chars);

    let path = "resources/monalisa.jpg";
    let image = image::open(path)?;
    log::debug!("Image loaded: {}", path);

    let font_data = std::fs::read("resources/NotoSansJP-Regular.otf")?;
    let mut m = Model::from_vec(font_data)?;
    log::debug!("Model created: {:?}", m);

    m.run(32, &chars, &image)?;

    View::run();

    Ok(())
}
