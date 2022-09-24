use winit::event::{Event, WindowEvent, ElementState};
use winit::event_loop::{ControlFlow, EventLoop};

use triangles::model::Model;
use triangles::model::TexFace;
use triangles::renderer::Renderer;
use triangles::teximg::Teximg;

fn color(name: char) -> [u8; 3] {
	match name {
		'r' => [255, 0, 0],
		'g' => [0, 255, 0],
		'b' => [0, 0, 255],
		'w' => [255, 255, 255],
		c => panic!("{}", c),
	}
}

fn rgb_to_16uv(rgb: [u8; 3]) -> [f32; 2] {
	let xr = (rgb[0] / 8) as f32 / 32.0;
	let xb = rgb[2] as f32 / 256.0 / 32.0;
	let xg = rgb[1] as f32 / 4.0;
	[xr + xb, xg]
}

fn main() {
	let el = EventLoop::new();
	let mut rdr = Renderer::new(&el);
	rdr.upload_tex(Teximg::preset_rgb565(), 0);
	let mut vs = vec![
		[000., 000., 0.0, 1.0],
		[200., 000., 0.0, 1.0],
		[400., 000., 0.0, 1.0],
		[400., 200., 0.0, 1.0],
		[400., 400., 0.0, 1.0],
		[200., 400., 0.0, 1.0],
		[000., 400., 0.0, 1.0],
		[000., 200., 0.0, 1.0],
	];
	let mut phase = 0f32;
	let uvs = vec![
		rgb_to_16uv(color('r')),
		rgb_to_16uv(color('g')),
		rgb_to_16uv(color('b')),
		rgb_to_16uv(color('w')),
	];
	eprintln!("{:?}", uvs);
	let fs = vec![
		TexFace { vid: [0, 7, 2], layer: 0, uvid: [0; 3] },
		TexFace { vid: [2, 1, 4], layer: 0, uvid: [1; 3] },
		TexFace { vid: [4, 3, 6], layer: 0, uvid: [2; 3] },
		TexFace { vid: [6, 5, 0], layer: 0, uvid: [3; 3] },
	];
	el.run(move |event, _, ctrl| match event {
		Event::WindowEvent { event: e, .. } => match e {
			WindowEvent::CloseRequested => {
				*ctrl = ControlFlow::Exit;
			}
			WindowEvent::Resized(_) => {
				rdr.damage();
			}
			WindowEvent::KeyboardInput { 
				input,
				..
			} => {
				if input.state == ElementState::Pressed {
					phase += 0.1;
					rdr.redraw();
				}
			}
			_ => {}
		},
		Event::RedrawRequested(_window_id) => {
			eprintln!("redraw");
			for idx in 0..8 {
				let angle = phase + idx as f32 / 8.0 * 2.0 * std::f32::consts::PI;
				vs[idx][2] = angle.sin() / 2.0 + 0.5;
			}
			let model = Model {
				vs: vs.clone(),
				uvs: uvs.clone(),
				tex_faces: fs.clone(),
				z: 0,
			};
			rdr.render2(&model);
		}
		Event::MainEventsCleared => {
			eprintln!("idle");
			*ctrl = ControlFlow::Wait;
		}
		_ => {}
	})
}
