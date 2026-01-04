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
pub struct Selector {
    selector: SelectorType
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
pub enum SelectorType {
    Tag { tag: String }
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

    /// Checks if the given `HtmlNode` matches the `SelectorType` of the current object.
    ///
    /// # Parameters
    /// - `node`: A reference to an `HtmlNode` which will be tested for a match.
    ///
    /// # Returns
    /// - `true` if the `HtmlNode` matches the selector criteria.
    /// - `false` otherwise.
    ///
    /// # Behavior
    /// - If the `SelectorType` is a `Tag` selector:
    ///   - If the `HtmlNode` is an `Element`, it checks whether the tag of the element equals the selector's tag.
    ///   - If the `HtmlNode` is a `Text` node, it will always return `false`.
    ///
    /// # Examples
    /// ```rust
    /// let selector = SelectorType::Tag { tag: "div".to_string() };
    /// let html_node = HtmlNode { node_type: HtmlNodeType::Element(HtmlElement { tag: "div".to_string() }) };
    /// assert!(selector.matches(&html_node));
    ///
    /// let text_node = HtmlNode { node_type: HtmlNodeType::Text("example".to_string()) };
    /// assert!(!selector.matches(&text_node));
    /// ```
    fn matches(&self, node: &HtmlNode) -> bool {
        match &self.selector {
            SelectorType::Tag { tag } => {
                match &node.node_type {
                    HtmlNodeType::Element(e) => {
                        e.tag == *tag
                    }
                    HtmlNodeType::Text(_) => {false}
                }
            }
        }
    }
}
