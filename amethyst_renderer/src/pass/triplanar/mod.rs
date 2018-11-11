pub use self::{interleaved::DrawTriplanar};

mod interleaved;

use pass::util::TextureType;

static VERT_SRC: &[u8] = include_bytes!("../shaders/vertex/basic.glsl");
static FRAG_SRC: &[u8] = include_bytes!("../shaders/fragment/triplanar.glsl");

static TEXTURES: [TextureType; 3] = [TextureType::Albedo, TextureType::Emission, TextureType::Normal];
