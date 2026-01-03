use std::collections::HashMap;

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

    /// Checks if the current character in the `style` string matches the given `literal` character,
    /// and increments the index if it matches. If the current character does not match or the index
    /// is out of bounds, it panics with a parsing error.
    ///
    /// # Arguments
    ///
    /// * `literal` - A `char` representing the expected literal to match against the current character
    /// in the `style` string.
    ///
    /// # Panics
    ///
    /// This function will panic if:
    /// - The current character in the `style` string does not match the provided `literal`.
    /// - `self.index` is out of bounds for the `style` string.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut parser = MyParser {
    ///     style: String::from("abc"),
    ///     index: 0,
    /// };
    /// parser.literal('a'); // Matches and increments the index
    /// parser.literal('b'); // Matches and increments the index
    /// parser.literal('x'); // Panics with "Parsing error: Expected literal 'x'"
    /// ```
    fn literal(&mut self, literal: char) {
        if !((self.style[self.index] == literal) && self.index < self.style.len()) {
            panic!("Parsing error: Expected literal '{}'", literal);
        }
        self.index += 1;
    }

    /// Parses a pair of words separated by a colon (`:`), while allowing
    /// optional whitespace around the colon. Each word must conform to
    /// the parsing logic defined by the `word()` method.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a tuple `(String, String)`, where:
    /// - The first `String` (property) is converted to lowercase.
    /// - The second `String` (value) remains in its original form.
    ///
    /// If parsing fails at any step, an `Err(String)` will be returned
    /// containing an error message.
    ///
    /// # Steps
    /// 1. Invokes the `word()` method to parse the first word (`prop`).
    /// 2. Consumes any additional whitespace after the first word.
    /// 3. Expects and consumes the `:` literal character.
    /// 4. Consumes any additional whitespace after the colon.
    /// 5. Invokes the `word()` method again to parse the second word (`val`).
    /// 6. Converts the first word (`prop`) to lowercase and returns the tuple.
    ///
    /// # Errors
    /// - If the `word()` method fails to parse either the property or value.
    /// - If the `literal(':')` check fails to match the expected colon.
    ///
    /// # Example
    /// ```rust
    /// let mut parser = MyParser::new("key : value");
    /// let result = parser.pair();
    /// assert_eq!(result, Ok(("key".to_string(), "value".to_string())));
    /// ```
    fn pair(&mut self) -> Result<(String, String), String> {
        let prop = self.word()?;
        self.whitespace();
        self.literal(':');
        self.whitespace();
        let val = self.word()?;
        Ok((prop.to_lowercase(), val))
    }

    /// Parses a series of key-value pairs from the input, storing them in a `HashMap`.
    ///
    /// This method iterates over the style data, extracting key-value pairs by
    /// repeatedly calling the `pair()` method. Each pair is followed by optional
    /// whitespace, a mandatory semicolon (`';'`), and more optional whitespace. The
    /// iteration continues until the end of the style input is reached.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a `HashMap<String, String>` with all the parsed
    /// key-value pairs if successful, or a `String` containing an error message if
    /// an error occurs during parsing.
    ///
    /// # Errors
    ///
    /// - Returns an `Err(String)` if the `pair()` method fails to parse a key-value pair.
    /// - Errors may also arise if the expected semicolon or other contextual parsing rules
    ///   are violated in the input.
    ///
    /// # Example
    /// ```
    /// let mut parser = YourParser::new("key1: value1; key2: value2;");
    /// let result = parser.body();
    /// assert!(result.is_ok());
    /// let map = result.unwrap();
    /// assert_eq!(map.get("key1").unwrap(), "value1");
    /// assert_eq!(map.get("key2").unwrap(), "value2");
    /// ```
    fn body(&mut self) -> Result<HashMap::<String, String>, String> {
        let mut pairs = HashMap::<String, String>::new();
        while (self.index < self.style.len()) {
            let pair = self.pair()?;
            pairs.insert(pair.0, pair.1);
            self.whitespace();
            self.literal(';');
            self.whitespace();
        }
        Ok(
            pairs
        )
    }




}