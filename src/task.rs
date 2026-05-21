pub struct Task {
    task_code: Option<Box<dyn FnOnce()>>
}

impl Task {
    pub fn new(task_code: impl FnOnce() + 'static) -> Self {
        Self {
            task_code: Some(Box::new(task_code)),
        }
    }

    pub fn run(&mut self) {
        if let Some(task_code) = self.task_code.take() {
            task_code();
        }
    }
}