use ab_glyph::PxScale;
use std::sync::LazyLock;

pub mod color;
pub mod correlation;
pub mod element;
pub mod model;
pub mod view;

const F64_ALMOST_ZERO: f64 = 1e-12;
const NUM_OF_CANDIDATES: usize = 16;
const IMAGE_FONT_SIZE: u32 = 18;
const IMAGE_MARGIN: u32 = 1;
const IMAGE_SIZE: u32 = IMAGE_FONT_SIZE + IMAGE_MARGIN * 2;
const FULL_WIDTH_SPACE: char = '　';
const PER_CHARACTER_DELAY_MS: u64 = 10;

static GLYPH_SCALE: LazyLock<PxScale> = LazyLock::new(|| PxScale::from(16.0));

pub const TYPESET: &str = include_str!("../assets/typeset.txt");
pub const FONT_DATA: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/", "NotoSansJP-Regular.otf"));
