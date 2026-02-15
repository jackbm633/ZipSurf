use std::cell::RefCell;
use std::rc::Rc;
use crate::tab::Tab;

/// A `Browser` structure that simulates a web browser with multiple tabs.
///
/// # Fields
///
/// * `tabs` - A vector of `Rc<RefCell<Tab>>`, representing the list of all tabs
///   currently open in the browser. Each tab is wrapped in a `Rc<RefCell<Tab>>`
///   to enable shared ownership and interior mutability.
///
/// * `current_tab` - An `Rc<RefCell<Tab>>` representing the currently active
///   tab in the browser. This allows the browser to keep track of the user's
///   interaction focus.
///
/// # Usage
///
/// This struct is intended to be used to manage and interact with multiple
/// browser tabs, enabling features such as switching between tabs, closing
/// tabs, or interacting with the contents of the current tab.
///
/// # Example
/// ```
/// use std::rc::Rc;
/// use std::cell::RefCell;
///
/// struct Tab {
///     url: String,
/// }
///
/// let first_tab = Rc::new(RefCell::new(Tab { url: String::from("https://example.com") }));
/// let second_tab = Rc::new(RefCell::new(Tab { url: String::from("https://rust-lang.org") }));
///
/// let mut browser = Browser {
///     tabs: vec![first_tab.clone(), second_tab.clone()],
///     current_tab: first_tab.clone(),
/// };
///
/// assert_eq!(browser.tabs.len(), 2);
/// assert_eq!(browser.current_tab.borrow().url, "https://example.com");
/// ```
pub struct Browser {
    tabs: Vec<Rc<RefCell<Tab>>>,
    current_tab: Rc<RefCell<Tab>>
}