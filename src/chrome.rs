use std::cell::RefCell;
use std::rc::{Rc, Weak};
use crate::browser::Browser;

pub struct Chrome {
    pub(crate) browser: Weak<RefCell<Browser>>
}
