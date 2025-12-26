
/// Represents a node in a document structure, which can either be a tag or plain text.
///
/// This enum is used to model the different types of elements that can exist in a document
/// or similar data structure.
///
/// # Variants
///
/// * `Tag(Tag)`
///   Represents an element with a tag, such as an HTML or XML element. The `Tag` type
///   encapsulates information related to the tag, such as the tag name, attributes, and children.
///
/// * `Text(Text)`
///   Represents a plain text element. The `Text` type contains the actual string data
///   for the text content.
pub enum Token {
    Tag(Tag),
    Text(Text)
}

/// A struct representing a tag.
///
/// The `Tag` struct is used to encapsulate a string-based tag.
/// It can be used to represent labels, keywords, or identifiers in various contexts.
///
/// # Fields
///
/// * `tag` - A `String` representing the value of the tag.
///
/// # Examples
///
/// ```
/// let my_tag = Tag {
///     tag: String::from("example"),
/// };
/// println!("{}", my_tag.tag); // Output: example
/// ```
/// struct
pub struct Tag {
    pub(crate) tag: String
}

/// ```rust
/// Struct `Text`
///
/// Represents a text entity containing a single field `text`.
///
/// # Fields
///
/// * `text` - A `String` that holds the content of the text.
///
/// # Example
///
/// ```
/// let my_text = Text {
///     text: String::from("Hello, world!"),
/// };
///
/// println!("{}", my_text.text); // Outputs: Hello, world!
/// ```
/// ```
pub struct Text {
    pub(crate) text: String
}
