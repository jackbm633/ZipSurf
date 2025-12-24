use std::{collections::HashMap, io::{BufRead, BufReader, Read, Write}, net::TcpStream};

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

    /// Executes a full HTTP GET request and returns the response body.
    ///
    /// This method handles the complete lifecycle of an HTTP/1.0 request:
    /// 1. **Connection**: Establishes a TCP link to port 80.
    /// 2. **Transmission**: Sends a formatted GET request.
    /// 3. **Status & Header Parsing**: Validates the server response and maps headers.
    /// 4. **Safety Check**: Rejects encoded or chunked transfer formats that 
    ///    require complex decompression/reassembly.
    /// 5. **Body Extraction**: Reads all remaining data from the stream into a String.
    ///
    /// # Returns
    /// * `Ok(String)` - The raw text/HTML content of the requested page.
    /// * `Err(String)` - If a network error occurs, or if the server uses unsupported encodings.
    pub fn request(&self) -> Result<String, String> {
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

            // Read the response headers.
            let mut response_headers: HashMap<String, String> = HashMap::new();
            
            loop {
                let mut header_line = String::new();
                reader.read_line(&mut header_line).map_err(
                    |e| format!("Failed to read header line: {}", e))?;
                header_line = header_line.to_owned().trim_end().to_string();
                if header_line.is_empty() {
                    break; // End of headers
                }
                if let Some((key, value)) = header_line.split_once(":") {
                    response_headers.insert(key.to_lowercase().to_string(), value.trim().to_string());
                }
            }
            // Read the remainder of the response body.
            if response_headers.contains_key("transfer-encoding")
                || response_headers.contains_key("content-encoding")
            {
                return Err("Unsupported transfer or content encoding".to_string());
            }

            let mut buf = String::new();
            reader.read_to_string(&mut buf).map_err(|error| error.to_string())?;

            
            Ok(buf)
        } else {
            Err(format!("Failed to connect to host {}", self.host).to_string())
        }
    }
}