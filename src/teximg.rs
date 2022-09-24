use image::ImageBuffer;

pub struct Teximg {
	pub dim: [u32; 2],
	// rgba8
	pub data: Vec<u8>,
}

pub type TexImage = ImageBuffer<image::Rgba<u8>, Vec<u8>>;

impl Teximg {
	pub fn from_image_buffer(image_buffer: TexImage) -> Self {
		let dim = image_buffer.dimensions();
		Self {
			dim: [dim.0, dim.1],
			data: image_buffer.into_vec(),
		}
	}

	pub fn preset_rgb565() -> Self {
		let image = ImageBuffer::from_fn(1024, 64, |x, y| {
			image::Rgba::from([
				(x / 32) as u8 * 8,
				y as u8 * 4,
				(x % 32) as u8 * 8,
				255,
			])
		});
		image.save("test.png");
		Self::from_image_buffer(image)
	}
}
