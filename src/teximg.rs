pub struct Teximg {
	pub dim: [u32; 2],
	// rgba8
	pub data: Vec<u8>,
}

pub type TexImage = image::ImageBuffer<image::Rgba<u8>, Vec<u8>>;

impl Teximg {
	pub fn from_image_buffer(image_buffer: TexImage) -> Self {
		let dim = image_buffer.dimensions();
		Self {
			dim: [dim.0, dim.1],
			data: image_buffer.into_vec(),
		}
	}
}
