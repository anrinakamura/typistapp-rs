use std::{io::Write, path::Path};

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let font_path = Path::new(&out_dir).join("NotoSansJP-Regular.otf");

    if !font_path.exists() {
        let url = "https://raw.githubusercontent.com/notofonts/noto-cjk/main/Sans/SubsetOTF/JP/NotoSansJP-Regular.otf";
        let response = ureq::get(url)
            .call()
            .expect("Failed to download font")
            .body_mut()
            .read_to_vec()
            .expect("Failed to read font data");

        let mut file = std::fs::File::create(&font_path).expect("Failed to create font file");
        file.write_all(&response)
            .expect("Failed to write font data to file");
    }
}
