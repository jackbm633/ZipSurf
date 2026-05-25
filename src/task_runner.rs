use std::rc::Rc;

use crate::task::Task;

pub struct TaskRunner {
    tab: Rc<RefCell<Tab>>,
    tasks: Vec<Task>
}

impl TaskRunner {
    pub fn schedule_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    pub fn run(&mut self) {
        if self.tasks.len() > 0 {
            let mut task = self.tasks.pop().unwrap();
            task.run();
        }
    }
}