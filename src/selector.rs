use std::cell::RefCell;
use std::rc::Rc;
use crate::node::{HtmlNode, HtmlNodeType};

/// A struct representing a generic selector used to encapsulate some selection mechanism.
///
/// This struct contains a single field `selector` of type `SelectorType`,
/// which is used to define the specific type of selection logic or data
/// associated with the selector.
///
/// # Fields
///
/// * `selector` - Represents the underlying selection logic or criteria.
///   The type `SelectorType` can be customized to define the specific
///   selection behavior.
///
/// # Example
///
/// ```rust
/// use your_crate::Selector;
///
/// let my_selector = Selector {
///     selector: SelectorType::ExampleType,
/// };
/// ```
///
/// Note: Replace `SelectorType::ExampleType` with the appropriate
/// implementation of `SelectorType`.
#[derive(Clone)]
pub struct Selector {
    pub(crate) selector: SelectorType
}

/// An enum representing different types of selectors that can be used
/// to query or match elements in a structured format (e.g., HTML, XML).
///
/// # Variants
///
/// - `Tag`:
///   Represents a selector based on a tag name. This is typically used
///   to match elements by their tag, such as `<div>` or `<span>`.
///
///   - `tag`:
///     A `String` representing the name of the tag to be matched.
///
/// # Example
///
/// ```rust
/// use your_crate::SelectorType;
///
/// let selector = SelectorType::Tag { tag: "div".to_string() };
/// if let SelectorType::Tag { tag } = selector {
///     println!("Matching tag: {}", tag);
/// }
/// ```
#[derive(Clone)]
pub enum SelectorType {
    Tag { tag: String },
    Descendant { ancestor: Box<Selector>, descendant: Box<Selector>}
}

impl Selector {
    /// Creates a new `Selector` with the specified tag.
    ///
    /// # Arguments
    ///
    /// * `tag` - A `String` representing the tag name to be associated with the `Selector`.
    ///
    /// # Returns
    ///
    /// A new instance of `Selector` configured with the specified tag.
    ///
    /// # Example
    ///
    /// ```rust
    /// let tag_selector = Selector::new_tag("div".to_string());
    /// ```
    fn new_tag(tag: String) -> Self {
        Selector { selector: SelectorType::Tag { tag }}
    }

    /// Determines if a given `HtmlNode` matches a specified selector.
    ///
    /// # Parameters
    /// - `self`: The current selector that is being used for matching.
    /// - `node`: A reference-counted `HtmlNode` wrapped in a `RefCell`. This represents
    ///   the node in the DOM tree that will be checked against the selector.
    ///
    /// # Returns
    /// - `bool`: Returns `true` if the provided `HtmlNode` matches the selector,
    ///   otherwise returns `false`.
    ///
    /// # Behavior
    /// - The behavior varies depending on the type of selector:
    ///   - `SelectorType::Tag`: Matches if the current `HtmlNode` is an element node
    ///     with a tag name equal to the provided tag in the selector. Returns `false`
    ///     if the node is a text node or the tag does not match.
    ///   - `SelectorType::Descendant`: Matches if the `descendant` selector matches
    ///     the current node and there exists an ancestor node in the DOM tree that
    ///     matches the `ancestor` selector. This works by traversing the parent chain
    ///     of the current node.
    ///
    /// # Note
    /// - For `SelectorType::Descendant`, if the `descendant` selector does not match
    ///   the current node, the function immediately returns `false`. Otherwise, it
    ///   iteratively traverses up the DOM tree by following the `parent` references
    ///   to check for a matching `ancestor`.
    ///
    /// # Example
    /// ```
    /// // Assuming `node` is an Rc<RefCell<HtmlNode>> and `selector` is a valid selector:
    /// if selector.matches(node) {
    ///     println!("The node matches the selector!");
    /// } else {
    ///     println!("The node does not match the selector.");
    /// }
    /// ```
    pub(crate) fn matches(&self, mut node: Rc<RefCell<HtmlNode>>) -> bool {
        match &self.selector {
            SelectorType::Tag { tag } => {
                match &node.borrow().node_type {
                    HtmlNodeType::Element(e) => {
                        e.tag == *tag
                    }
                    HtmlNodeType::Text(_) => {false}
                }
            }
            SelectorType::Descendant { descendant, ancestor } => {
                if !descendant.matches(node.clone()) {return false}
                while node.borrow().parent.is_some() {
                    if ancestor.matches(node.borrow().parent.clone().unwrap()) {
                        return true
                    }
                    let new_node = node.borrow().parent.clone().unwrap();
                    node = new_node;
                }
                false
            }
        }
    }
    
    /// Computes the priority of the current `SelectorType`.
    ///
    /// # Returns
    /// * An `i32` representing the priority of the selector:
    ///   - If the selector is of type `Tag`, it returns a fixed priority of `1`.
    ///   - If the selector is of type `Descendant`, it recursively computes the priority
    ///     by summing the priorities of its `ancestor` and `descendant` selectors.
    ///
    /// # Behavior
    /// This function matches the `SelectorType` of the current instance:
    /// - For `SelectorType::Tag`, a fixed priority of `1` is assigned.
    /// - For `SelectorType::Descendant`, the priority is calculated based on the priorities
    ///   of the `ancestor` and `descendant` selectors.
    ///
    /// # Examples
    /// ```rust
    /// let tag_selector = Selector::new(SelectorType::Tag { name: "div".to_string() });
    /// assert_eq!(tag_selector.priority(), 1);
    ///
    /// let descendant_selector = Selector::new(SelectorType::Descendant {
    ///     ancestor: Box::new(tag_selector),
    ///     descendant: Box::new(Selector::new(SelectorType::Tag { name: "span".to_string() })),
    /// });
    /// assert_eq!(descendant_selector.priority(), 2); // 1 (ancestor) + 1 (descendant)
    /// ```
    pub(crate) fn priority(&self) -> i32 {
        match &self.selector {
            SelectorType::Tag { .. } => {1}
            SelectorType::Descendant { ancestor, descendant } => {
                ancestor.priority() + descendant.priority()
            }
        }
    }
}
