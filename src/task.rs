use crate::tab::Tab;

pub struct Task {
    task_code: Option<Box<dyn FnOnce(&Tab) + Send>>
}

impl Task {
pub fn new(task_code: impl FnOnce(&Tab) + Send + 'static) -> Self {
        Self {
            task_code: Some(Box::new(task_code)),
        }
    }
    pub fn run(&mut self, tab: &Tab) {
        if let Some(task_code) = self.task_code.take() {
            task_code(tab);
        }
    }
}