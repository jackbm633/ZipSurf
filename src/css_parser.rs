/// Represents a CSS parser for processing and analyzing CSS stylesheets.
///
/// The `CssParser` struct holds a reference to a CSS style string and an index to
/// track the current parsing position. This struct is designed to facilitate the
/// parsing of CSS rules, selectors, and properties.
///
/// # Fields
///
/// - `style`: A reference to a string slice containing the CSS styles. This is the CSS
///   source that the parser processes. The lifetime `'b` ensures that the `CssParser`
///   does not outlive the provided CSS string.
/// - `index`: An `i64` representing the current position within the CSS string. It is used
///   to track the parsing progress in the string.
///
/// # Example
///
/// ```rust
/// let css_content = "body { color: black; }";
/// let parser = CssParser {
///     style: css_content,
///     index: 0,
/// };
/// println!("Parsing CSS: {}", parser.style);
/// ```
///
/// The `CssParser` can be extended to include methods for traversing, finding, or extracting
/// specific CSS rules or properties.
struct CssParser<'b> {
    style: &'b str,
    index: i64,
}

impl<'a> CssParser<'a> {
    /// Creates a new instance of the struct.
    ///
    /// # Parameters
    /// - `styles`: A string slice that defines the styles to be associated with the new instance.
    ///
    /// # Returns
    /// Returns a new instance of the struct with the given `styles` and initializes the `index` to `0`.
    ///
    /// # Example
    /// ```rust
    /// let styles = "bold, italic";
    /// let instance = CssParser::new(styles);
    /// ```
    pub fn new(styles: &'a str) -> Self {
        Self {
            style: styles,
            index: 0
        }
    }


}