use winit::event::{ElementState, Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use triangles::model::Model;
use triangles::model::TexFace;
use triangles::renderer::Renderer;
use triangles::bmtext::FontConfig;

fn main() {
	// initialize
	let el = EventLoop::new();
	let mut rdr = Renderer::new(&el);

	// draw text
	let mut fc = FontConfig::default();
	fc.resize_screen(rdr.get_size());
	let img = fc.bitw_loader("../bitw/data/lat15_terminus32x16.txt");
	rdr.upload_tex(img, 0);
	let vs = fc.generate_vs();
	let uvs = fc.generate_uvs();
	let tex_faces = fc.text2fs("hello,world", 0);
	let model = Model { vs, uvs, tex_faces };
	rdr.insert_model(0, &model);
	rdr.set_z(0, 1);

	// draw triangle
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
			layer: -1, // no texture
			uvid: [0; 3],
		}],
	};
	rdr.insert_model(1, &model);

	// event loop
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
