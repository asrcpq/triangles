use winit::event_loop::{ControlFlow, EventLoop};
use winit::event::{Event, WindowEvent};
use image::ImageBuffer;

use triangles::model::Model;
use triangles::model::SolidFace;
use triangles::renderer::Renderer;

fn main() {
	let el = EventLoop::new();
	let mut rdr = Renderer::new(
		vec![ImageBuffer::from_fn(
			1024, 1024, |_, _| image::Rgba::from([0, 0, 0, 255]),
		)],
		&el,
	);
	el.run(move |event, _, ctrl| match event {
		Event::WindowEvent {event: e, ..} => match e {
			WindowEvent::CloseRequested => {
				*ctrl = ControlFlow::Exit;
			},
			WindowEvent::Resized(_) => {
				rdr.damage();
			},
			WindowEvent::KeyboardInput {..} => {
				let vs = vec![
					[000., 000., 0.0, 1.0],
					[000., 200., 0.0, 1.0],
					[200., 000., 0.0, 1.0],
					[200., 200., 0.0, 1.0],
				];
				let f1 = SolidFace {
					vid: [0, 1, 2],
					rgba: [1.0, 0.0, 0.0, 1.0],
				};
				let f2 = SolidFace {
					vid: [1, 2, 3],
					rgba: [0.0, 0.0, 1.0, 1.0],
				};
				let model = Model {
					vs,
					uvs: Vec::new(),
					tex_faces: Vec::new(),
					solid_faces: vec![f1, f2],
				};
				rdr.render2(&model);
			},
			_ => {},
		},
		_ => {},
	})
}
