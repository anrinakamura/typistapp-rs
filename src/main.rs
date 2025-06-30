use anyhow::Result;
use typistapp::model::Model;

fn main() {
    env_logger::init();
    println!("Hello, world!");
}

#[allow(dead_code)]
fn run() -> Result<()> {
    let font_data = std::fs::read("path/to/font.ttf")?;
    let _ = Model::from_vec(font_data)?;
    Ok(())
}
