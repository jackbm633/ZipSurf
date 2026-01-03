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
struct CssParser<> {
    style: Vec<char>,
    index: usize,
}

impl CssParser {
    /// Creates a new instance of the struct with the provided styles.
    ///
    /// # Arguments
    ///
    /// * `styles` - A string slice representing the styles to be stored in the instance.
    ///
    /// # Returns
    ///
    /// * A new instance of the struct initialized with the provided styles, where:
    ///   - `style` is a `Vec<char>` containing individual characters from the input `styles`.
    ///   - `index` is initialized to `0`.
    ///
    /// # Example
    ///
    /// ```rust
    /// let styles = "bold italic";
    /// let instance = MyStruct::new(styles);
    /// assert_eq!(instance.style, vec!['b', 'o', 'l', 'd', ' ', 'i', 't', 'a', 'l', 'i', 'c']);
    /// assert_eq!(instance.index, 0);
    /// ```
    pub fn new(styles: &str) -> Self {
        Self {
            style: styles.chars().collect(),
            index: 0
        }
    }

    /// Skips over all consecutive whitespace characters in the `style` field starting
    /// from the current `index` position. Updates the `index` field to point to the
    /// first non-whitespace character or the end of the `style` field.
    ///
    /// # Behavior
    /// - Iterates through the `style` field character by character, starting at the
    ///   current `index`.
    /// - Checks if each character is a whitespace character using `.is_whitespace()`.
    /// - Increments the `index` for each whitespace character encountered.
    /// - Stops when a non-whitespace character is found or the end of the `style` field
    ///   is reached.
    ///
    /// # Parameters
    /// This function operates on a mutable reference to `self`.
    ///
    /// # Example
    /// ```rust
    /// let mut parser = Parser {
    ///     style: "   example",
    ///     index: 0,
    /// };
    /// parser.whitespace();
    /// assert_eq!(parser.index, 3); // Skips the leading whitespaces
    /// ```
    ///
    /// # Notes
    /// - Assumes that `self.style` is a valid string slice and `index` is properly
    ///   initialized within its bounds to avoid panics.
    /// - Does not modify the `style` field, only the `index`.
    fn whitespace(&mut self) {
        while self.index < self.style.len() && self.style[self.index].is_whitespace() {
            self.index += 1;
        }
    }

    /// Extracts a single word from the `style` field, starting at the current position of `self.index`.
    ///
    /// A word is defined as a sequence of alphanumeric characters or the characters `#`, `-`, `.`, `%`.
    /// The function iterates through the `style` slice starting from `self.index`, collecting characters
    /// that match the word criteria until it encounters whitespace or an invalid character.
    ///
    /// # Returns
    ///
    /// - `Ok(String)` containing the extracted word if parsing is successful.
    /// - `Err(String)` with an error message if no valid word is found at the current starting position.
    ///
    /// # Errors
    ///
    /// Returns an error in the following scenarios:
    /// - If there is no valid word at the current position (`self.index`).
    ///
    /// # Example
    ///
    /// ```
    /// let mut parser = SomeParser {
    ///     style: vec!['h', 'e', 'l', 'l', 'o', ' ', 'w', 'o', 'r', 'l', 'd'],
    ///     index: 0,
    /// };
    ///
    /// // Parsing the first word
    /// let word = parser.word().unwrap();
    /// assert_eq!(word, "hello");
    ///
    /// // Move the index past the whitespace
    /// parser.index += 1;
    ///
    /// // Parsing the next word
    /// let second_word = parser.word().unwrap();
    /// assert_eq!(second_word, "world");
    /// ```
    ///
    /// # Assumptions
    ///
    /// - The `style` property is a slice of characters (`Vec<char>`).
    /// - The `index` field tracks the current parsing position in the `style` slice.
    fn word(&mut self) -> Result<String, String> {
        let start = self.index;
        while self.index < self.style.len() && !self.style[self.index].is_whitespace() {
            if self.style[self.index].is_alphanumeric() ||
                "#-.%".contains(self.style[self.index]) {
                self.index += 1;
            } else {
                break;
            }
        }
        if self.index <= start {
            return Err("Parsing error: Expected word".to_string());
        }
        Ok(self.style[start..self.index].iter().collect())
    }




}