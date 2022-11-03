use winit::event::{Event, WindowEvent, ElementState};
use winit::event_loop::{ControlFlow, EventLoop};

use triangles::model::Model;
use triangles::model::TexFace;
use triangles::renderer::Renderer;

fn main() {
	let el = EventLoop::new();
	let mut rdr = Renderer::new(&el);
	let vs = vec![
		[000., 000., 1.0, 1.0],
		[200., 000., 0.0, 1.0],
		[400., 000., 1.0, 1.0],
		[400., 200., 0.0, 1.0],
		[400., 400., 1.0, 1.0],
		[200., 400., 0.0, 1.0],
		[000., 400., 1.0, 1.0],
		[000., 200., 0.0, 1.0],
	];
	let mut phase = 0u32;
	let fs = vec![
		TexFace { vid: [0, 7, 2], color: [1.0, 0.0, 0.0, 1.0], layer: -1, uvid: [0; 3] },
		TexFace { vid: [2, 1, 4], color: [0.0, 1.0, 0.0, 1.0], layer: -1, uvid: [0; 3] },
		TexFace { vid: [4, 3, 6], color: [0.0, 0.0, 1.0, 1.0], layer: -1, uvid: [0; 3] },
		TexFace { vid: [6, 5, 0], color: [1.0, 0.0, 1.0, 1.0], layer: -1, uvid: [0; 3] },
	];
	for (idx, face) in fs.into_iter().enumerate() {
		let model = Model {
			vs: vs.clone(),
			uvs: vec![],
			tex_faces: vec![face],
		};
		rdr.insert_model(idx as u32, &model);
	}
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
					phase += 1;
					rdr.set_visibility(phase % 4, phase % 3 == 2);
					rdr.redraw();
				}
			}
			e => {eprintln!("{:?}", e)}
		},
		Event::RedrawRequested(_window_id) => {
			eprintln!("redraw");
			rdr.render2();
		}
		Event::MainEventsCleared => {
			eprintln!("idle");
			*ctrl = ControlFlow::Wait;
		}
		_ => {}
	})
}
