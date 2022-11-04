use winit::event::{ElementState, Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use triangles::model::cmodel::{Model, TexFace};
use triangles::renderer::Renderer;
use triangles::bmtext::FontConfig;
use triangles::camcon::Camcon;

fn main() {
	// initialize
	let el = EventLoop::new();
	let mut rdr = Renderer::new(&el);
	let ssize = rdr.get_size();

	// draw text
	let mut fc = FontConfig::default();
	fc.resize_screen(ssize);
	let img = fc.bitw_loader("../bitw/data/lat15_terminus32x16.txt");
	rdr.upload_tex(img, 0);
	let mut model = fc.generate_model();
	model.tex_faces = fc.text2fs("hello,world", 0);
	let mut text_model = rdr.insert_model(&model);
	text_model.set_z(1);

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
	let _triangle_model = rdr.insert_model(&model);

	let mut camcon = Camcon::new(ssize);

	// event loop
	el.run(move |event, _, ctrl| match event {
		Event::WindowEvent { event: e, .. } => {
			if camcon.process_event(&e) {
				rdr.redraw();
			}
			match e {
				WindowEvent::CloseRequested => {
					*ctrl = ControlFlow::Exit;
				}
				WindowEvent::Resized(_) => {
					let ssize = rdr.get_size();
					camcon.resize(ssize);
					rdr.damage();
				}
				WindowEvent::KeyboardInput { input, .. } => {
					if input.state == ElementState::Pressed {
						rdr.redraw();
					}
				}
				_ => {}
			}
		}
		Event::RedrawRequested(_window_id) => {
			rdr.render(camcon.get_camera());
		}
		Event::MainEventsCleared => {
			*ctrl = ControlFlow::Wait;
		}
		_ => {}
	})
}
