use std::collections::HashMap;

/// Represents a CSS parser that processes a vector of characters to parse CSS styles.
///
/// The `CssParser` struct contains the following fields:
///
/// - `style`: A `Vec<char>` that holds the characters of the CSS style to be parsed.
/// - `index`: A `usize` value that represents the current position of the parser within the `style` vector.
///
/// # Example
///
/// ```rust
/// let css_string = "body { color: black; }".chars().collect();
/// let mut parser = CssParser {
///     style: css_string,
///     index: 0,
/// };
///
/// // Example usage of the parser
/// assert_eq!(parser.style[parser.index], 'b');
/// ```
pub struct CssParser {
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

    /// ```rust
    /// Attempts to match the current character in the `style` string with the provided literal.
    ///
    /// This function checks if the current character at the `index` position in the `style` vector
    /// matches the provided `literal` character. If it matches, the `index` is incremented. If it
    /// doesn't match or the index is out of bounds, an error is returned.
    ///
    /// # Arguments
    ///
    /// * `literal` - A `char` representing the literal character to match against the current
    ///               character in the `style` vector.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the current character matches the given literal and the index is successfully advanced.
    /// * `Err(String)` if the current character does not match the given literal or the index is out of bounds.
    ///                The error contains a message indicating the expected literal.
    ///
    /// # Errors
    ///
    /// Returns an error in the following cases:
    /// - The `index` exceeds the bounds of the `style` string.
    /// - The current character in the `style` string does not match the provided `literal`.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut parser = Parser {
    ///     style: vec!['a', 'b', 'c'],
    ///     index: 0,
    /// };
    ///
    /// assert_eq!(parser.literal('a'), Ok(()));
    /// assert_eq!(parser.literal('b'), Ok(()));
    /// assert_eq!(parser.literal('x'), Err("Parsing error: Expected literal 'x'".to_string()));
    /// ```
    ///
    /// # Assumptions
    ///
    /// This function assumes the `self.style` field is a `Vec<char>` and `self.index` is a valid position or
    /// potential position in the vector.
    /// ```
    fn literal(&mut self, literal: char) -> Result<(), String> {
        if !(self.index < self.style.len() && (self.style[self.index] == literal)) {
            return Err(format!("Parsing error: Expected literal '{}'", literal))
        }
        self.index += 1;

        Ok(())
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

    /// Parses and constructs a `HashMap<String, String>` by iterating over the input data
    /// and extracting key-value pairs.
    ///
    /// # Returns
    /// * `Ok(HashMap<String, String>)` - A map containing parsed key-value pairs if the parsing is successful.
    /// * `Err(String)` - An error message indicating what went wrong during parsing if an unrecoverable error occurs.
    ///
    /// # Errors
    /// This function may return an `Err` in the following cases:
    /// - The `pair()` method encounters an unrecoverable error during key-value pair extraction.
    /// - The `literal(';')` method fails to parse the expected delimiter (e.g., `;`) and error handling
    ///   determines that the parsing process should terminate.
    ///
    /// # Behavior
    /// - The function repeatedly attempts to parse key-value pairs until the input data is exhausted
    ///   or an unrecoverable error causes the function to terminate early.
    /// - If an error occurs during the extraction of a pair (`pair()`), or while ensuring
    ///   the presence of a delimiter (`literal(';')`), the `handle_body_error()` method
    ///   is invoked to determine whether parsing should continue or terminate.
    /// - Whitespace around key-value pairs and delimiters is handled and ignored using
    ///   the `whitespace()` method.
    ///
    /// # Implementation Details
    /// - Calls to `self.pair()` extract a single key-value pair. If this extraction fails (`pair.is_err()`),
    ///   the `handle_body_error()` method determines whether to continue parsing or exit the loop.
    /// - Successfully parsed key-value pairs are added to the `HashMap` using `insert`.
    /// - Delimiters (e.g., `;`) are validated after each key-value pair using the `literal(';')` method.
    /// - Whitespace is consumed at appropriate points to allow flexible parsing of the input format.
    ///
    /// # Notes
    /// - The function assumes that the instance's `self` contains fields like `index`, `style`, and methods
    ///   such as `pair()`, `handle_body_error()`, `whitespace()`, and `literal()`, necessary for parsing logic.
    /// - Designed to work with data structures or parsers specific to the caller's context.
    ///
    /// # Example
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// // Assuming `self` is appropriately defined and initialized
    /// let result: Result<HashMap<String, String>, String> = self.body();
    ///
    /// match result {
    ///     Ok(pairs) => {
    ///         for (key, value) in pairs {
    ///             println!("Key: {}, Value: {}", key, value);
    ///         }
    ///     },
    ///     Err(err) => {
    ///         eprintln!("Failed to parse body: {}", err);
    ///     }
    /// }
    /// ```
    pub(crate) fn body(&mut self) -> Result<HashMap<String, String>, String> {
        let mut pairs = HashMap::<String, String>::new();
        while self.index < self.style.len() {
            let pair = self.pair();
            if pair.is_err() {
                let should_break = self.handle_body_error();
                if should_break {
                    break;
                }
            }
            let pair_unwrapped = pair?;
            pairs.insert(pair_unwrapped.0, pair_unwrapped.1);
            self.whitespace();
            if self.literal(';').is_err() {
                let should_break = self.handle_body_error();
                if should_break {
                    break;
                }
            }
            self.whitespace();
        }
        Ok(
            pairs
        )
    }

    /// Handles errors encountered in parsing a body of code.
    ///
    /// This function attempts to recover from a parsing error by skipping
    /// over tokens until a specific delimiter (`';'`) is encountered. Upon
    /// finding the delimiter, it consumes it and performs necessary cleanup
    /// (e.g., handling surrounding whitespace).
    ///
    /// # Returns
    /// - `false`: If the recovery process is successful (delimiter found and handled).
    /// - `true`: If recovery could not proceed (delimiter not found).
    ///
    /// # Panics
    /// The function will panic if the expected delimiter (`';'`) is not found
    /// after calling the `self.literal(';')` method. The panic message is
    /// currently a placeholder and should be replaced with a more descriptive one.
    ///
    /// # Implementation Details
    /// - The function uses `self.ignore_until` to skip over tokens until a `';'`
    ///   is encountered. It then processes the delimiter using `self.literal(';')`
    ///   and removes any surrounding whitespace with `self.whitespace()`.
    ///
    /// # Examples
    /// ```
    /// let mut parser = Parser::new();
    /// let success = parser.handle_body_error();
    /// assert_eq!(success, false); // If recovery succeeded with the `;` delimiter
    /// ```
    fn handle_body_error(&mut self) -> bool {
        let why = self.ignore_until(vec![';']);
        if (why == Some(';')) {
            self.literal(';').expect("TODO: panic message");
            self.whitespace();
            return false;
        }
        true
    }

    /// Advances the internal index until one of the specified characters (`literals`)
    /// is found in the `style` string. If a matching character is encountered, it is returned.
    /// If none of the characters in `literals` are found by the time the end of the `style`
    /// string is reached, the method returns `None`.
    ///
    /// # Arguments
    ///
    /// * `literals` - A vector of characters to look for in the `style` string.
    ///
    /// # Returns
    ///
    /// * `Option<char>` - The first matching character wrapped in `Some` if found,
    ///   or `None` if no match is encountered.
    ///
    /// # Behavior
    ///
    /// * The method increments `self.index` as it loops through the characters in the `style` string.
    /// * If the end of the `style` string is reached before finding a matching character,
    ///   `None` is returned.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut parser = YourStruct { style: "abcde".to_string(), index: 0 };
    /// let result = parser.ignore_until(vec!['c', 'e']);
    /// assert_eq!(result, Some('c'));
    /// assert_eq!(parser.index, 2); // `index` stops at the position of the match
    ///
    /// let result = parser.ignore_until(vec!['x', 'y']);
    /// assert_eq!(result, None);
    /// assert_eq!(parser.index, 5); // `index` reaches the end of the string
    /// ```
    fn ignore_until(&mut self, literals: Vec<char>) -> Option<char> {
        while self.index < self.style.len() {
            if literals.contains(&self.style[self.index]) {
                return Some(self.style[self.index]);
            } else {
                self.index += 1;
            }
        }
        None
    }



}

