/// ```rust
/// A struct representing a URL with its scheme (protocol).
///
/// `Url` is used to store and represent the scheme (e.g., "http", "https", "ftp")
/// component of a URL. Additional components of a URL can be included by extending
/// this struct with more fields.
///
/// # Fields
/// - `scheme` (`String`): The scheme or protocol of the URL (e.g., "http", "https").
///
/// # Examples
///
/// ```rust
/// let url = Url {
///     scheme: String::from("https"),
/// };
/// println!("Scheme: {}", url.scheme); // Output: Scheme: https
/// ```
/// ```
struct Url {
    scheme: String
}

impl Url {
    /// ```rust
    /// Creates a new `Url` instance from the given URL string.
    ///
    /// This function parses the given URL to extract its scheme and validates
    /// that the scheme is `"http"`. If the scheme is not `"http"`, the function
    /// will panic.
    ///
    /// # Arguments
    ///
    /// * `url` - A string slice representing the URL to be parsed. The URL provided
    ///           must include the scheme (e.g., `"http://example.com"`).
    ///
    /// # Returns
    ///
    /// Returns a new `Url` instance with the extracted scheme.
    ///
    /// # Panics
    ///
    /// This function will panic if:
    /// * The input `url` does not contain a valid scheme separated by `"://"`.
    /// * The extracted scheme is not `"http"`.
    ///
    /// # Examples
    ///
    /// ```
    /// let url = Url::new("http://example.com");
    /// assert_eq!(url.scheme, "http".to_string());
    /// ```
    ///
    /// Note: This function currently assumes only HTTP URLs are supported.
    /// ```
    fn new(url: &str) -> Self {
        // Extract the scheme, which is separated by the URL by ://.
        // Browser currently only supports http so let's check that too.
        let url_split = url.splitn(2, "://").collect::<Vec<_>>();
        let scheme = url_split[0];
        let url = url_split[1];
        assert_eq!(scheme, "http");

        return Url {scheme: scheme.to_string() }
    }
}