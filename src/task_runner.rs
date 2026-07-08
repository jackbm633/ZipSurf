use crate::tab::TabMessage;
use crate::task::Task;

/// A task runner that manages and executes scheduled tasks.
/// 
/// Sends tasks to the tab's main thread event loop.
pub struct TaskRunner {
    pub(crate) task_tx: std::sync::mpsc::Sender<TabMessage>,
}

impl TaskRunner {
    pub fn new(task_tx: std::sync::mpsc::Sender<TabMessage>) -> Self {
        Self { task_tx }
    }

    /// Schedules a task to be run.
    /// 
    /// # Arguments
    /// * `task` - The task to schedule
    pub fn schedule_task(&mut self, task: Task) {
        let _ = self.task_tx.send(TabMessage::RunTask(task));
    }

    pub fn run(&mut self) {
        // Obsolete: Use Tab's event loop thread instead.
    }
}

