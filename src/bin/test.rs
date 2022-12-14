use winit::event::{ElementState, Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use triangles::bmtext::FontConfig;
use triangles::camcon::Camcon;
use triangles::model::cmodel::{Face, Model};
use triangles::renderer::Renderer;
use triangles::teximg::Teximg;

fn main() {
	// initialize
	let el = EventLoop::new();
	let mut rdr = Renderer::new(&el);
	let ssize = rdr.get_size();

	// draw text
	let img = Teximg::load("../fontdata/v1/unifont1.png");
	let fc = FontConfig::new(ssize, img.dim, [16, 16]);
	rdr.upload_tex(img, 0);
	let mut model = fc.generate_model();
	model.faces = Vec::new();
	model.faces.extend(fc.text2fs(
		[0, 0],
		"hello,锟斤拷".chars(),
		[1.0, 1.0, 1.0, 1.0],
		0,
	));
	model.faces.extend(fc.text2fs(
		[0, 1],
		"world烫烫烫".chars(),
		[0.0, 1.0, 1.0, 1.0],
		0,
	));
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
		faces: vec![Face {
			vid: [0, 1, 2],
			color: [0.5, 0.0, 0.0, 1.0],
			layer: -1, // no texture
			uvid: [0; 3],
		}],
	};
	let _triangle_model = rdr.insert_model(&model);

	let mut camcon = Camcon::new(ssize);
	let mut dirty = false;

	// event loop
	el.run(move |event, _, ctrl| match event {
		Event::WindowEvent { event: e, .. } => {
			if camcon.process_event(&e) {
				dirty = true;
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
						dirty = true;
					}
				}
				_ => {}
			}
		}
		Event::RedrawRequested(_window_id) => {
			rdr.render(camcon.get_camera());
		}
		Event::MainEventsCleared => {
			if dirty {
				dirty = false;
				rdr.redraw();
			}
			*ctrl = ControlFlow::Wait;
		}
		_ => {}
	})
}
