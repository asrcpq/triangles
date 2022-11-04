use crate::model::cmodel::{Model, TexFace};
use crate::teximg::Teximg;

#[derive(Default)]
pub struct FontConfig {
	font_size: [u32; 2],
	screen_size: [u32; 2],
	texture_size: [u32; 2],
}

impl FontConfig {
	pub fn resize_screen(&mut self, new_size: [u32; 2]) {
		self.screen_size = new_size;
		eprintln!("FontConfig resized: {:?}", new_size);
	}

	pub fn get_font_size(&self) -> [u32; 2] {
		self.font_size
	}

	pub fn bitw_loader(&mut self, path: &str) -> Teximg {
		let string = std::fs::read_to_string(path).unwrap();
		let mut tmp_num: i32 = -1;
		let mut row: u32 = 0;
		let mut col: u32 = 0;
		let mut data = vec![vec![[0u8; 4]; 1024]; 1024];
		let mut tmp_data: Vec<Vec<bool>> = Vec::new();
		for line in string.split('\n') {
			let line = line.trim_end();
			if line.is_empty() {
				if tmp_num == -1 {
					continue;
				}
				// check row/col
				if row == 0 {
					row = tmp_data.len() as u32;
				} else {
					assert_eq!(row, tmp_data.len() as u32);
				}
				if col == 0 {
					col = tmp_data[0].len() as u32;
				}

				// write content
				let chars_per_row = 1024 / col;
				let chpos_x = (tmp_num as u32) % chars_per_row;
				let pos_x = (chpos_x * col) as usize;
				let chpos_y = (tmp_num as u32) / chars_per_row;
				let pos_y = (chpos_y * row) as usize;
				for x in 0..col as usize {
					for y in 0..row as usize {
						data[pos_y + y][pos_x + x] = if tmp_data[y][x] {
							[255; 4]
						} else {
							[0; 4]
						};
					}
				}

				tmp_num = -1;
				tmp_data = Vec::new();
				continue;
			}
			if tmp_num == -1 {
				tmp_num = line.parse::<i32>().unwrap();
				continue;
			}
			let bools = line.chars().map(|x| x == '1').collect();
			tmp_data.push(bools);
		}
		let dim = [1024, 1024];
		self.texture_size = dim;
		self.font_size = [col, row];
		Teximg {
			dim,
			data: data
				.into_iter()
				.flat_map(|x| x.into_iter().flat_map(|x| x.into_iter()))
				.collect(),
		}
	}

	fn generate_vs(&self) -> Vec<[f32; 4]> {
		let [xx, yy] = self.get_terminal_size_in_char();
		let [col, row] = self.font_size;
		let mut vs = Vec::new();
		for y in 0..=yy {
			for x in 0..=xx {
				vs.push([(x * col) as f32, (y * row) as f32, 0.0, 1.0]);
			}
		}
		vs
	}

	fn generate_uvs(&self) -> Vec<[f32; 2]> {
		let [xx, yy] = self.get_texture_size_in_char();
		let [tx, ty] = self.texture_size;
		let [col, row] = self.font_size;
		let mut uvs = Vec::new();
		for y in 0..=yy {
			for x in 0..=xx {
				uvs.push([(x * col) as f32 / tx as f32, (y * row) as f32 / ty as f32])
			}
		}
		uvs
	}

	pub fn generate_model(&self) -> Model {
		let vs = self.generate_vs();
		let uvs = self.generate_uvs();
		Model {vs, uvs, tex_faces: Default::default()}
	}

	fn get_terminal_size_in_char(&self) -> [u32; 2] {
		[
			self.screen_size[0] / self.font_size[0],
			self.screen_size[1] / self.font_size[1],
		]
	}

	fn get_texture_size_in_char(&self) -> [u32; 2] {
		[
			self.texture_size[0] / self.font_size[0],
			self.texture_size[1] / self.font_size[1],
		]
	}

	pub fn text2fs(&self, text: &str, layer: i32) -> Vec<TexFace> {
		// x1 terminal size(in char), x2 texture size(in char)
		let [x1, _] = self.get_terminal_size_in_char();
		let [x2, _] = self.get_texture_size_in_char();
		let x1 = x1 as usize;
		let x2 = x2 as usize;
		let mut result = Vec::new();
		for (idx, ch) in text.bytes().enumerate() {
			// 10 chars has 11 vertices
			let pos_x = idx % x1;
			let pos_y = idx / x1;
			let screen_leftup = pos_y * (x1 + 1) + pos_x;
			let screen_leftdown = (pos_y + 1) * (x1 + 1) + pos_x;
	
			let pos_x = (ch as usize) % x2;
			let pos_y = (ch as usize) / x2;
			let texture_leftup = pos_y * (x2 + 1) + pos_x;
			let texture_leftdown = (pos_y + 1) * (x2 + 1) + pos_x;
	
			let face1 = TexFace {
				vid: [screen_leftup, screen_leftup + 1, screen_leftdown],
				color: [0.0; 4],
				layer,
				uvid: [texture_leftup, texture_leftup + 1, texture_leftdown],
			};
			let face2 = TexFace {
				vid: [screen_leftup + 1, screen_leftdown, screen_leftdown + 1],
				color: [0.0; 4],
				layer,
				uvid: [texture_leftup + 1, texture_leftdown, texture_leftdown + 1],
			};
			result.push(face1);
			result.push(face2);
		}
		result
	}
}
