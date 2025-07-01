use std::io::{BufRead, BufReader};

use anyhow::Result;
use typistapp::model::Model;

fn main() {
    env_logger::init();
    println!("Hello, world!");
}

#[allow(dead_code)]
fn run() -> Result<()> {
    let reader = BufReader::new(std::fs::File::open("resources/typeset.txt")?);
    let mut chars = vec![];
    for line in reader.lines() {
        chars.extend(line?.chars());
    }

    let font_data = std::fs::read("resources/NotoSansJP-Regular.otf")?;
    let _ = Model::from_vec(font_data)?;
    Ok(())
}
