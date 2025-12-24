use std::{io::{BufRead, BufReader, Write}, net::TcpStream};

/// Represents a decomposed HTTP URL.
/// 
/// This structure holds the individual components of a URL after it has been 
/// validated and parsed by the [`Url::new`] constructor.
pub struct Url {
    /// The protocol used (currently limited to "http").
    pub scheme: String,
    /// The domain name or IP address of the server.
    pub host: String,
    /// The resource location on the server (the part following the first forward slash).
    pub path: String,
}

impl Url {
    /// Creates a new `Url` instance by parsing a string slice.
    ///
    /// This parser decomposes a URL into three parts: scheme, host, and path.
    /// Currently, it only supports the `http` scheme.
    ///
    /// # Arguments
    ///
    /// * `url` - A string slice containing the URL to be parsed (e.g., "http://example.com/index.html").
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` - A `Url` struct containing the parsed components.
    /// * `Err(String)` - An error message if the URL format is invalid or the scheme is unsupported.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// 1. The string does not contain the `://` separator.
    /// 2. The scheme is anything other than `http`.
    ///
    /// # Examples
    ///
    /// ```
    /// let my_url = Url::new("[http://google.com/search](http://google.com/search)").unwrap();
    /// assert_eq!(my_url.host, "google.com");
    /// assert_eq!(my_url.path, "/search");
    /// ```
    pub fn new(url: &str) -> Result<Self, String> {
        // Extract the scheme, which is separated by the URL by ://.
        // Browser currently only supports http so let's check that too.
        let url_split = url.splitn(2, "://").collect::<Vec<_>>();
        if url_split.len() != 2 {
            return Err("URL missing :// separator".to_string());
        }
        let scheme = url_split[0];
        let mut url_remaining: String = url_split[1].to_string();
        if scheme != "http" {
            return Err(format!("Unsupported URL scheme: {}", scheme));
        }
        
        // Now, we separate the host from the path. The host comes before the first /,
        // and the path is everything after it.
        if !url_remaining.contains("/")
        {
            url_remaining.push('/');
        }

        let host_path = url_remaining.splitn(2, "/").collect::<Vec<_>>();
        let host = host_path[0];
        let path = "/".to_owned() +  if host_path.len() > 1  {host_path[1]} else {""};



        Ok(
            Url {
                scheme: scheme.to_string(),
                host: host.to_string(),
                path: path
            }
        )
    }

    /// Sends an HTTP GET request and parses the server's status response.
    ///
    /// This method handles the initial handshake of the HTTP protocol:
    /// 1. Transmits the request over TCP.
    /// 2. Reads the first line of the response (the Status Line).
    /// 3. Validates the status line format (e.g., "HTTP/1.0 200 OK").
    ///
    /// # Returns
    /// * `Ok(())` - If the request was sent and a valid HTTP status line was received.
    /// * `Err(String)` - If the connection fails, the request fails to send, 
    ///   or the server returns a malformed response.
    pub fn request(&self) -> Result<(), String> {
        // Connect to the host on port 80
        if let Ok(mut stream) = TcpStream::connect(format!("{}:80", self.host)) {
            // Clone the stream and create a buffered reader for it to read the response later
            let clone_stream = stream.try_clone().map_err(
                |e| format!("Failed to clone stream: {}", e))?;
            let mut reader = BufReader::new(clone_stream); 
            let request = format!("GET {} HTTP/1.0\r\nHost: {}\r\n\r\n", self.path, self.host);
            let request_result = stream.write_all(request.as_bytes());
            if let Err(e) = request_result {
                return Err(format!("Failed to send request: {}", e));
            }

            let mut status_line = String::new();
            reader.read_line(&mut status_line).map_err(
                |e| format!("Failed to read response: {}", e))?;
            let status_parts: Vec<&str> = status_line.splitn(3, ' ').collect();
            if status_parts.len() < 3 {
                return Err("Malformed HTTP response".to_string());
            }

            let version = status_parts[0];
            let status_code = status_parts[1];
            let status_text = status_parts[2].trim_end();


            Ok(())
        } else {
            Err(format!("Failed to connect to host {}", self.host).to_string())
        }
    }
}