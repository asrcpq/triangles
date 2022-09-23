use winit::event_loop::{ControlFlow, EventLoop};
use winit::event::{Event, WindowEvent};
use image::ImageBuffer;

use triangles::model::Model;
use triangles::model::SolidFace;
use triangles::teximg::Teximg;
use triangles::renderer::Renderer;

fn main() {
	let el = EventLoop::new();
	let mut rdr = Renderer::new(&el);
	let image = ImageBuffer::from_fn(1024, 1024,
		|_, _| image::Rgba::from([0, 0, 0, 255])
	);
	rdr.upload_tex(Teximg::from_image_buffer(image), 0);
	let mut vs = vec![
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
	el.run(move |event, _, ctrl| match event {
		Event::WindowEvent {event: e, ..} => match e {
			WindowEvent::CloseRequested => {
				*ctrl = ControlFlow::Exit;
			},
			WindowEvent::Resized(_) => {
				rdr.damage();
			},
			WindowEvent::KeyboardInput {..} => {
				vs.iter_mut().for_each(|x| x[0] += 10.0);
				rdr.redraw();
			},
			_ => {},
		},
		Event::RedrawRequested(_window_id) => {
			eprintln!("redraw");
			let model = Model {
				vs: vs.clone(),
				uvs: Vec::new(),
				tex_faces: Vec::new(),
				solid_faces: vec![f1.clone(), f2.clone()],
			};
			rdr.render2(&model);
		}
		Event::MainEventsCleared => {
			eprintln!("idle");
			*ctrl = ControlFlow::Wait;
		}
		_ => {},
	})
}
