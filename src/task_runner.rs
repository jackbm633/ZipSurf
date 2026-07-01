use std::{cell::RefCell, rc::Rc, sync::{Arc, RwLock}};

use crate::{tab::Tab, task::Task};

/// A task runner that manages and executes scheduled tasks.
/// 
/// Maintains a queue of tasks and executes them one at a time.
pub struct TaskRunner {
    pub tab: Arc<RwLock<Tab>>,
    pub tasks: Vec<Task>,
    pub condvar: Arc<(std::sync::Mutex<bool>, std::sync::Condvar)>
}

impl TaskRunner {
    /// Schedules a task to be run.
    /// 
    /// # Arguments
    /// * `task` - The task to schedule
    pub fn schedule_task(&mut self, task: Task) {
        // Acquire the lock to ensure thread safety when modifying the task queue.
        let lock = self.condvar.0.lock().unwrap();
        self.tasks.push(task);
        // Notify any waiting threads that a new task has been scheduled.
        self.condvar.1.notify_all();
        // Release the lock after modifying the task queue.
        drop(lock)    
    }

    /// Runs the next scheduled task in the queue.
    /// 
    /// If there are no tasks, this method does nothing.

    pub fn run(&mut self) {
        // 1. Drain any tasks sent from background threads
        let mut bg_tasks = Vec::new();
        {
            let tab_borrow = self.tab.read().unwrap();
            if let Some(ref rx) = tab_borrow.task_rx {
                while let Ok(task) = rx.try_recv() {
                    bg_tasks.push(task);
                }
            }
        } // tab_borrow is dropped here

        // 2. Execute the background tasks on the main thread
        for mut task in bg_tasks {
            let tab_ref = self.tab.read().unwrap();
            task.run(&tab_ref);
        }

        // 3. Execute normal queue tasks
        let mut task: Option<Task> = None;
        let lock = self.condvar.0.lock().unwrap();
        if self.tasks.len() > 0 {
            task = Some(self.tasks.pop().unwrap());
        }
        drop(lock);

        if let Some(mut t) = task {
            let tab_ref = self.tab.read().unwrap();
            t.run(&tab_ref);
        }
    }
}
