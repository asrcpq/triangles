use image::ImageBuffer;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use triangles::model::Model;
use triangles::model::{SolidFace, TexFace};
use triangles::renderer::Renderer;
use triangles::teximg::Teximg;

fn main() {
	let el = EventLoop::new();
	let mut rdr = Renderer::new(&el);
	let image = ImageBuffer::from_fn(1024, 1024, |x, y| {
		image::Rgba::from([x as u8, y as u8, (x + y) as u8, 255])
	});
	rdr.upload_tex(Teximg::from_image_buffer(image), 0);
	let mut vs = vec![
		[000., 000., 0.0, 1.0],
		[000., 200., 0.0, 1.0],
		[200., 000., 0.0, 1.0],
		[200., 200., 0.0, 1.0],
	];
	let uvs = vec![[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]];
	let f1 = SolidFace {
		vid: [0, 1, 2],
		rgba: [1.0, 0.0, 0.0, 1.0],
	};
	let f2 = TexFace {
		vid: [1, 2, 3],
		uvid: [0, 1, 2],
		layer: 0,
	};
	el.run(move |event, _, ctrl| match event {
		Event::WindowEvent { event: e, .. } => match e {
			WindowEvent::CloseRequested => {
				*ctrl = ControlFlow::Exit;
			}
			WindowEvent::Resized(_) => {
				rdr.damage();
			}
			WindowEvent::KeyboardInput { .. } => {
				vs.iter_mut().for_each(|x| x[0] += 10.0);
				rdr.redraw();
			}
			_ => {}
		},
		Event::RedrawRequested(_window_id) => {
			eprintln!("redraw");
			let model = Model {
				vs: vs.clone(),
				uvs: uvs.clone(),
				tex_faces: vec![f2.clone()],
				solid_faces: vec![f1.clone()],
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
