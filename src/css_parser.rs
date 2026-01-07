use std::collections::HashMap;
use crate::selector::Selector;
use crate::selector::SelectorType::{Descendant, Tag};

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
        self.literal(':')?;
        self.whitespace();
        let val = self.word()?;
        Ok((prop.to_lowercase(), val))
    }


    /// Parses and extracts key-value pairs from a structured style body until
    /// the closing brace (`}`) is encountered or an error is handled.
    ///
    /// This method processes the contents of the `self.style` field starting
    /// from the current position (`self.index`) and returns a `HashMap` containing
    /// the key-value pairs. If parsing errors occur, they are handled using the
    /// `handle_body_error` function, which determines whether parsing should terminate.
    ///
    /// # Returns
    /// - `Ok(HashMap<String, String>)`: A hashmap containing the extracted key-value pairs.
    /// - `Err(String)`: An error message if parsing fails unrecoverably.
    ///
    /// # Behavior
    /// - Processes until the end of the body (`'}`) or until an unrecoverable error occurs.
    /// - Key-value pairs are separated by semicolons (`;`), and the method skips any
    ///   whitespace between pairs or after the semicolons.
    /// - Calls `self.pair` to parse each key-value pair.
    /// - If `self.pair` or `self.literal(';')` returns an error, the error is handled
    ///   using `self.handle_body_error`.
    ///
    /// # Error Handling
    /// - If an error occurs during pair extraction or semicolon parsing,
    ///   `self.handle_body_error` determines if the error is recoverable. If
    ///   unrecoverable, parsing terminates and the current state is returned.
    ///
    /// # Examples
    /// ```
    /// # use std::collections::HashMap;
    /// # use your_module::YourStruct;
    /// let mut parser = YourStruct::new("{key1: value1; key2: value2;}");
    /// let result = parser.body();
    /// assert!(result.is_ok());
    ///
    /// let map = result.unwrap();
    /// assert_eq!(map.get("key1"), Some(&"value1".to_string()));
    /// assert_eq!(map.get("key2"), Some(&"value2".to_string()));
    /// ```
    pub(crate) fn body(&mut self) -> Result<HashMap<String, String>, String> {
        let mut pairs = HashMap::<String, String>::new();
        while self.index < self.style.len() && self.style[self.index] != '}' {
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

    /// Handles errors encountered while parsing the body of a construct.
    ///
    /// This method attempts to recover from a parsing error by ignoring tokens until
    /// one of the specified delimiters (`;` or `}`) is encountered. If the delimiter `;`
    /// is found, the method consumes it, skips any subsequent whitespace, and signals
    /// that the error recovery process completed without further issues. If no such
    /// delimiter is found (or if the delimiter is `}`), the method determines that it
    /// cannot recover further.
    ///
    /// # Returns
    /// * `false` - if the parsing error was recovered successfully and a `;` was found.
    /// * `true` - if recovery was impossible (or if `}` was encountered).
    ///
    /// # Panics
    /// This method will panic with a "TODO: panic message" if the `literal` method
    /// fails to consume and match the `;` character.
    ///
    /// # Example
    /// ```rust
    /// let mut parser = Parser::new();
    /// assert!(parser.handle_body_error()); // Example usage
    /// ```
    fn handle_body_error(&mut self) -> bool {
        let why = self.ignore_until(vec![';', '}']);
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

    /// Parses a CSS-like selector from the current style string and constructs a `Selector` object with
    /// either a single tag or a descendant hierarchy of tags.
    ///
    /// # Details
    /// This function builds a hierarchical representation of a selector by repeatedly parsing individual
    /// tags and constructing a `Selector` object. The hierarchy constructed represents descendant
    /// relationships in the selector (e.g., for selectors such as `div span`, `div` is the ancestor of `span`).
    ///
    /// The parsing stops when a `{` character is encountered or when the end of the style is reached. Any
    /// unnecessary whitespace in the selector string is ignored.
    ///
    /// # Errors
    /// If the function encounters an error in parsing the selector, it will return a `Result::Err` with
    /// an appropriate error message.
    ///
    /// # Returns
    /// Returns a `Result`:
    /// - `Ok(Selector)` containing the parsed selector.
    /// - `Err(String)` containing an error message in case of parsing failure.
    ///
    /// # Example
    /// ```rust
    /// // Assume `self` is configured with a proper style string and methods.
    /// let selector = self.selector();
    /// match selector {
    ///     Ok(result) => println!("Parsed selector: {:?}", result),
    ///     Err(err) => eprintln!("Error parsing selector: {}", err),
    /// }
    /// ```
    ///
    /// # Prerequisites
    /// - The `self.word()` function must be able to properly parse valid tags.
    /// - The `self.whitespace()` function must handle whitespace clearing in the input.
    ///
    /// # Notes
    /// - The function assumes that `self.style` holds the input style string, and `self.index` tracks the
    ///   current parsing position.
    /// - Each tag in the hierarchy is converted to lowercase to ensure case-insensitive matching.
    ///
    /// # Dependencies
    /// This function depends on the following structures/classes:
    /// - `Selector`: Represents the selector being constructed.
    /// - `Tag { tag: String }`: Represents an individual tag in the selector.
    /// - `Descendant { ancestor: Box<Selector>, descendant: Box<Selector> }`: Represents a descendant
    ///   relationship between two selectors.
    ///
    /// See the implementation of `Selector`, `Tag`, and `Descendant` for more details.
    fn selector(&mut self) -> Result<Selector, String> {
        let mut out = Selector { selector: Tag {tag: self.word()?.to_lowercase()} };
        self.whitespace();
        while self.index < self.style.len() && self.style[self.index] != '{' {
            let tag = self.word()?;
            let descendant = Selector { selector: Tag {tag: tag.to_lowercase()} };
            out = Selector { selector: Descendant {ancestor: Box::from(out),
                descendant: Box::from(descendant)}}
        }
        Ok(out)
    }

    /// Parses a set of rules from the internal `style` representation and returns the result.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<(Selector, HashMap<String, String>)>)` - A vector of tuples where each tuple contains:
    ///   - A `Selector` representing a CSS selector.
    ///   - A `HashMap<String, String>` representing a collection of style properties for the selector.
    /// * `Err(String)` - A string specifying the reason for the failure during parsing.
    ///
    /// # Behavior
    ///
    /// This method iterates through the `style` data, attempting to parse CSS-style rules.
    /// It repeatedly calls an internal parsing method (`parse_internal`) to extract selectors and style
    /// properties. If parsing fails during this process:
    /// - It attempts to recover by skipping content until the next '}' character.
    /// - If no recovery point is found, it halts further parsing and returns the rules collected so far.
    ///
    /// After successfully parsing a rule, it expects a closing '}' character and processes any whitespace following it.
    ///
    /// # Errors
    ///
    /// If the `literal` method fails or the parsing logic encounters an unrecoverable issue,
    /// the method will return an error containing a descriptive string.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut parser = Parser::new("a { color: red; } b { font-size: 16px; }");
    /// let parsed_rules = parser.parse();
    /// assert!(parsed_rules.is_ok());
    /// let rules = parsed_rules.unwrap();
    /// assert_eq!(rules.len(), 2);
    /// ```
    pub fn parse(&mut self) -> Result<Vec<(Selector, HashMap<String, String>)>, String> {
        let mut rules = Vec::<(Selector, HashMap<String, String>)>::new();
        while self.index < self.style.len() {
            let result = self.parse_internal(&mut rules);
            match result {
                Ok(_) => (),
                Err(_err) =>  {
                    let why = self.ignore_until(Vec::from(['}']));
                    if why == Some('}') {
                        self.literal('}')?;
                        self.whitespace();
                    } else {
                        break
                    }
                }
            }
        }
        Ok(rules)
    }

    /// Parses a CSS-like rule and adds it to the provided vector of rules.
    ///
    /// This function processes input to extract a CSS selector and its associated
    /// style declarations, then appends them together as a tuple to the provided
    /// vector. The input is parsed in the following steps:
    ///
    /// 1. Skips leading whitespace characters.
    /// 2. Parses the selector using the `selector` method.
    /// 3. Verifies and consumes the opening '{' literal.
    /// 4. Skips any additional whitespace.
    /// 5. Extracts the body (key-value style declarations) using the `body` method.
    /// 6. Verifies and consumes the closing '}' literal.
    /// 7. Pushes the parsed `(selector, body)` pair into the `rules` vector.
    ///
    /// # Parameters
    ///
    /// - `rules`: A mutable reference to a vector of tuples, where each tuple
    ///   consists of a `Selector` (representing the selector for a rule) and a
    ///   `HashMap<String, String>` (representing the style key-value declarations).
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Indicates successful parsing and appending to the rules vector.
    /// - `Err(String)`: Returns an error message if parsing fails at any step.
    ///
    /// # Errors
    ///
    /// An error will be returned if any of the following parsing steps fail:
    /// - Selector parsing (`self.selector` returns an error).
    /// - Literal checking (`self.literal('{')` or `self.literal('}')` fails).
    /// - Body parsing (`self.body` returns an error).
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut parser = MyParser::new();
    /// let mut rules = Vec::new();
    ///
    /// match parser.parse_internal(&mut rules) {
    ///     Ok(_) => {
    ///         println!("Successfully parsed the rule!");
    ///         println!("{:?}", rules);
    ///     }
    ///     Err(e) => {
    ///         eprintln!("Failed to parse rule: {}", e);
    ///     }
    /// }
    /// ```
    ///
    /// # Notes
    ///
    /// This method assumes that `self.whitespace`, `self.selector`, `self.literal`,
    /// and `self.body` are implemented appropriately and will handle their
    /// respective tasks with robustness.
    ///
    /// # Dependencies
    ///
    /// - `Selector`: A type that represents a CSS selector.
    /// - `HashMap<String, String>`: The data structure used to store key-value
    ///   pairs of style declarations.
    fn parse_internal(&mut self, rules: &mut Vec<(Selector, HashMap<String, String>)>) -> Result<(), String> {
        self.whitespace();
        let selector = self.selector()?;
        self.literal('{')?;
        self.whitespace();
        let body = self.body()?;
        self.literal('}')?;
        rules.push((selector, body));
        Ok(())
    }
}

