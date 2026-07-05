use crate::js_context::JsContext;
use std::sync::Arc;

pub struct Task {
    task_code: Option<Box<dyn FnOnce(Arc<JsContext>) + Send>>
}

impl Task {
    pub fn new(task_code: impl FnOnce(Arc<JsContext>) + Send + 'static) -> Self {
        Self {
            task_code: Some(Box::new(task_code)),
        }
    }
    pub fn run(&mut self, js: Arc<JsContext>) {
        if let Some(task_code) = self.task_code.take() {
            task_code(js);
        }
    }
}