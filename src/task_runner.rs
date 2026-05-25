use std::{cell::RefCell, rc::Rc};

use crate::{tab::Tab, task::Task};

/// A task runner that manages and executes scheduled tasks.
/// 
/// Maintains a queue of tasks and executes them one at a time.
pub struct TaskRunner {
    pub tab: Rc<RefCell<Tab>>,
    pub tasks: Vec<Task>
}

impl TaskRunner {
    /// Schedules a task to be run.
    /// 
    /// # Arguments
    /// * `task` - The task to schedule
    pub fn schedule_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    /// Runs the next scheduled task in the queue.
    /// 
    /// If there are no tasks, this method does nothing.
    pub fn run(&mut self) {
        if self.tasks.len() > 0 {
            let mut task = self.tasks.pop().unwrap();
            task.run();
        }
    }
}