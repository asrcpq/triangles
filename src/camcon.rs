use winit::event::{ElementState, MouseButton, WindowEvent};

type V2 = nalgebra::Vector2<f32>;

// 2d camera controller
pub struct Camcon {
	world_center: V2,
	screen_r: V2, // e.g. (960.0, 540.0)
	zoom: f32,

	control_state: ControlState,
}

#[derive(Default)]
struct ControlState {
	pub move_button: bool,
	pub zoom_button: bool,
	pub prev_cursor_pos: Option<V2>,
}

impl Camcon {
	pub fn new(screen_size: [u32; 2]) -> Self {
		let r = V2::new(screen_size[0] as f32, screen_size[1] as f32) / 2.;
		Self {
			world_center: r,
			screen_r: r,
			zoom: 1.0,

			control_state: Default::default(),
		}
	}

	pub fn move_view(&mut self, ds: V2) {
		self.world_center -= ds / self.zoom;
	}

	pub fn s2w(&self, pos: V2) -> V2 {
		let result = (pos - self.screen_r) / self.zoom;
		result + self.world_center
	}

	pub fn resize(&mut self, new_size: [u32; 2]) {
		self.screen_r[0] = new_size[0] as f32 / 2.;
		self.screen_r[1] = new_size[1] as f32 / 2.;
	}

	pub fn zoom(&mut self, k: f32) {
		self.zoom *= k;
	}

	pub fn get_camera(&self) -> crate::M4 {
		let [cx, cy]: [f32; 2] = self.world_center.into();
		let [rx, ry]: [f32; 2] =
			[self.screen_r[0] / self.zoom, self.screen_r[1] / self.zoom];
		crate::M4::new_orthographic(
			cx - rx,
			cx + rx,
			cy - ry,
			cy + ry,
			1.0,
			-1.0,
		)
	}

	pub fn process_event(&mut self, event: &WindowEvent) -> bool {
		let mut result = false;
		match event {
			WindowEvent::CursorMoved { position, .. } => {
				let pos = V2::new(position.x as f32, position.y as f32);
				if !self.control_state.move_button {
					self.control_state.prev_cursor_pos = None;
					return false;
				}
				if let Some(prev_pos) =
					self.control_state.prev_cursor_pos.take()
				{
					if self.control_state.zoom_button {
						self.zoom(((pos - prev_pos).y / -100.0).exp());
					} else {
						self.move_view(pos - prev_pos);
					}
					result = true;
				}
				self.control_state.prev_cursor_pos = Some(pos);
			}
			WindowEvent::MouseInput { state, button, .. } => {
				if *button == MouseButton::Middle {
					self.control_state.move_button =
						*state == ElementState::Pressed;
				}
			}
			WindowEvent::ModifiersChanged(state) => {
				self.control_state.zoom_button = state.ctrl();
			}
			_ => {}
		}
		result
	}
}
