use image::ImageBuffer;
use winit::event::{Event, WindowEvent, ElementState};
use winit::event_loop::{ControlFlow, EventLoop};

use triangles::model::Model;
use triangles::model::{
	SolidFace,
//	TexFace,
};
use triangles::renderer::Renderer;
use triangles::teximg::Teximg;

fn color(name: char) -> [f32; 4] {
	match name {
		'r' => [1.0, 0.0, 0.0, 1.0],
		'g' => [0.0, 1.0, 0.0, 1.0],
		'b' => [0.0, 0.0, 1.0, 1.0],
		'w' => [1.0, 1.0, 1.0, 1.0],
		't' => [0.0; 4],
		c => panic!("{}", c),
	}
}

fn main() {
	let el = EventLoop::new();
	let mut rdr = Renderer::new(&el);
	let image = ImageBuffer::from_fn(1024, 1024, |x, y| {
		image::Rgba::from([x as u8, y as u8, (x + y) as u8, 255])
	});
	rdr.upload_tex(Teximg::from_image_buffer(image), 0);
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
	let fs = vec![
		SolidFace { vid: [0, 7, 2], rgba: color('r') },
		SolidFace { vid: [2, 1, 4], rgba: color('g') },
		SolidFace { vid: [4, 3, 6], rgba: color('b') },
		SolidFace { vid: [6, 5, 0], rgba: color('w') },
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
				uvs: vec![],
				tex_faces: vec![],
				solid_faces: fs.clone(),
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
