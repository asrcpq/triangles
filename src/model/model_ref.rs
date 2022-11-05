use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use super::compiled_model::CompiledModel;

#[derive(Clone)]
pub struct ModelRef {
	data: Rc<RefCell<CompiledModel>>,
}

impl ModelRef {
	pub fn new(compiled_model: CompiledModel) -> Self {
		Self {
			data: Rc::new(RefCell::new(compiled_model)),
		}
	}

	pub fn set_z(&mut self, z: i32) {
		self.data.borrow_mut().z = z;
	}

	pub fn set_visibility(&mut self, visible: bool) {
		self.data.borrow_mut().visible = visible;
	}

	pub fn dropped(&self) -> bool {
		Rc::strong_count(&self.data) <= 1
	}

	pub fn borrow(&self) -> Ref<CompiledModel> {
		self.data.borrow()
	}

	pub fn borrow_mut(&self) -> RefMut<CompiledModel> {
		self.data.borrow_mut()
	}
}
