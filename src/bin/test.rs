use winit::event::{ElementState, Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use triangles::model::Model;
use triangles::model::TexFace;
use triangles::renderer::Renderer;
use triangles::teximg::Teximg;

fn bitw_loader(path: &str) -> (Teximg, [usize; 2]) {
	let string = std::fs::read_to_string(path).unwrap();
	let mut tmp_num: i32 = -1;
	let mut row = 0;
	let mut col = 0;
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
				row = tmp_data.len();
			} else {
				assert_eq!(row, tmp_data.len());
			}
			if col == 0 {
				col = tmp_data[0].len();
			}

			// write content
			let chars_per_row = 1024 / col;
			let chpos_x = (tmp_num as usize) % chars_per_row;
			let pos_x = chpos_x * col;
			let chpos_y = (tmp_num as usize) / chars_per_row;
			let pos_y = chpos_y * row;
			for x in 0..col {
				for y in 0..row {
					data[pos_y + y][pos_x + x] = if tmp_data[y][x] {
						[255; 4]
					} else {
						[255, 0, 0, 32]
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
	let result = Teximg {
		dim: [1024, 1024],
		data: data
			.into_iter()
			.flat_map(|x| x.into_iter().flat_map(|x| x.into_iter()))
			.collect(),
	};
	(result, [col, row])
}

// xy1 terminal size(in char), xy2 texture size(in char)
fn text2fs(text: &str, x1: usize, x2: usize, layer: i32) -> Vec<TexFace> {
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

fn main() {
	let el = EventLoop::new();
	let mut rdr = Renderer::new(&el);
	let (img, [col, row]) = bitw_loader("../bitw/data/lat15_terminus32x16.txt");
	rdr.upload_tex(img, 0);
	let mut vs = Vec::new();
	for y in 0..=6 {
		for x in 0..=6 {
			vs.push([(x * col) as f32, (y * row) as f32, 0.0, 1.0]);
		}
	}
	let texture_char_per_row = 1024 / col;
	let texture_rows = 1024 / row;
	let mut uvs = Vec::new();
	for y in 0..=texture_rows {
		for x in 0..=texture_char_per_row {
			uvs.push([(x * col) as f32 / 1024.0, (y * row) as f32 / 1024.0])
		}
	}
	let tex_faces = text2fs("hello,world", 6, texture_char_per_row, 0);
	let model = Model { vs, uvs, tex_faces };
	rdr.insert_model(0, &model);
	rdr.set_z(0, 1);
	let model = Model {
		vs: vec![
			[0.0, 0.0, 0.5, 1.0],
			[30.0, 100.0, 0.0, 1.0],
			[100.0, 30.0, 0.0, 1.0],
		],
		uvs: vec![[0.0; 2]],
		tex_faces: vec![TexFace {
			vid: [0, 1, 2],
			color: [0.0, 0.0, 1.0, 1.0],
			layer: -1,
			uvid: [0; 3],
		}],
	};
	rdr.insert_model(1, &model);
	el.run(move |event, _, ctrl| match event {
		Event::WindowEvent { event: e, .. } => match e {
			WindowEvent::CloseRequested => {
				*ctrl = ControlFlow::Exit;
			}
			WindowEvent::Resized(_) => {
				rdr.damage();
			}
			WindowEvent::KeyboardInput { input, .. } => {
				if input.state == ElementState::Pressed {
					// rdr.insert_model(0, &model);
					rdr.redraw();
				}
			}
			_ => {}
		},
		Event::RedrawRequested(_window_id) => {
			eprintln!("redraw");
			rdr.render2();
		}
		Event::MainEventsCleared => {
			*ctrl = ControlFlow::Wait;
		}
		_ => {}
	})
}
