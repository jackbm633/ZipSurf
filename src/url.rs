use std::{
    collections::HashMap,
    fmt::Debug,
    io::{BufRead, BufReader, Read, Write},
    net::TcpStream,
};

use native_tls::TlsConnector;

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
    /// The port number for the connection (default is 80 for HTTP).
    pub port: u16,
}

impl Url {
    /// Fully parses a URL string into a structured `Url` object.
    ///
    /// This constructor handles scheme validation, host/port separation, 
    /// and path normalization. It is designed to prepare the data for 
    /// both TCP connection and HTTP request formatting.
    ///
    /// # Arguments
    /// * `url` - The raw URL string (e.g., "http://localhost:8080/index.html").
    ///
    /// # Returns
    /// * `Ok(Self)` - A `Url` instance with a "clean" host and explicit port.
    /// * `Err(String)` - If the URL is missing protocol separators or has an invalid port.
    pub fn new(url: &str) -> Result<Self, String> {
        // Extract the scheme, which is separated by the URL by ://.
        // Browser currently only supports http so let's check that too.
        let url_split = url.splitn(2, "://").collect::<Vec<_>>();
        if url_split.len() != 2 {
            return Err("URL missing :// separator".to_string());
        }
        let scheme = url_split[0];
        let mut url_remaining: String = url_split[1].to_string();
        if scheme != "http" && scheme != "https" {
            return Err(format!("Unsupported URL scheme: {}", scheme));
        }

        // Now, we separate the host from the path. The host comes before the first /,
        // and the path is everything after it.
        if !url_remaining.contains("/") {
            url_remaining.push('/');
        }

        let host_path = url_remaining.splitn(2, "/").collect::<Vec<_>>();
        let host = host_path[0];
        let path = "/".to_owned()
            + if host_path.len() > 1 {
                host_path[1]
            } else {
                ""
            };

        Ok(Url {
            scheme: scheme.to_string(),
            host: if host.contains(':') {
                host.splitn(2, ':').collect::<Vec<_>>()[0].to_string()
            } else {
                host.to_string()
            },
            path: path,
            port: if host.contains(':') {
                host.splitn(2, ':')
                    .collect::<Vec<_>>()[1]
                    .parse::<u16>()
                    .map_err(|_| "Invalid port number".to_string())?
            } else if scheme == "http" {
                80
            } else {
                443
            },
        })
    }

    /// Executes a secure or insecure HTTP GET request based on the URL scheme.
    ///
    /// This method handles protocol negotiation:
    /// 1. **Transport Layer**: Establishes a raw TCP connection.
    /// 2. **Security Layer**: If the scheme is `https`, it performs a TLS handshake
    ///    to encrypt the session.
    /// 3. **Abstraction**: Uses a trait object (`Box<dyn ReadWrite>`) to handle
    ///    both encrypted and unencrypted streams interchangeably.
    /// 4. **HTTP Transaction**: Sends the GET request and parses the response.
    ///
    /// # Returns
    /// * `Ok(String)` - The decrypted response body.
    /// * `Err(String)` - If the connection, TLS handshake, or parsing fails.
    pub fn request(&self) -> Result<String, String> {
        // Connect to the host on port 80
        if let Ok(tcp_stream) = TcpStream::connect(format!("{}:{}", self.host, self.port)) {
            let mut stream: Box<dyn ReadWrite> = if self.scheme == "https" {
                let connector = TlsConnector::new()
                    .map_err(|e| format!("Failed to create TLS connector: {}", e))?;
                let tls_stream = connector
                    .connect(&self.host, tcp_stream)
                    .map_err(|e| format!("Failed to establish TLS connection: {}", e))?;
                Box::new(tls_stream)
            } else {
                Box::new(tcp_stream)
            };

            let request = format!("GET {} HTTP/1.0\r\nHost: {}\r\n\r\n", self.path, self.host);
            let request_result = stream.write_all(request.as_bytes());
            if let Err(e) = request_result {
                return Err(format!("Failed to send request: {}", e));
            }

            let mut reader: BufReader<Box<dyn ReadWrite>> = BufReader::new(stream);

            let mut status_line = String::new();
            reader
                .read_line(&mut status_line)
                .map_err(|e| format!("Failed to read response: {}", e))?;
            let status_parts: Vec<&str> = status_line.splitn(3, ' ').collect();
            if status_parts.len() < 3 {
                return Err("Malformed HTTP response".to_string());
            }

            let _version = status_parts[0];
            let _status_code = status_parts[1];
            let _status_text = status_parts[2].trim_end();

            // Read the response headers.
            let mut response_headers: HashMap<String, String> = HashMap::new();

            loop {
                let mut header_line = String::new();
                reader
                    .read_line(&mut header_line)
                    .map_err(|e| format!("Failed to read header line: {}", e))?;
                header_line = header_line.to_owned().trim_end().to_string();
                if header_line.is_empty() {
                    break; // End of headers
                }
                if let Some((key, value)) = header_line.split_once(":") {
                    response_headers
                        .insert(key.to_lowercase().to_string(), value.trim().to_string());
                }
            }
            // Read the remainder of the response body.
            if response_headers.contains_key("transfer-encoding")
                || response_headers.contains_key("content-encoding")
            {
                return Err("Unsupported transfer or content encoding".to_string());
            }

            let mut buf = String::new();
            reader
                .read_to_string(&mut buf)
                .map_err(|error| error.to_string())?;

            Ok(buf)
        } else {
            Err(format!("Failed to connect to host {}", self.host).to_string())
        }
    }
    
    pub fn resolve(&self, mut url: &mut str) -> Result<Url, String> {
        if url.contains("://") {
            return Url::new(url);
        }
        if url.starts_with("//") {
            return Url::new(&format!("{}:{}", self.scheme, url));
        }

        let resolved_path = if url.starts_with('/') {
            url.to_string()
        } else {
            let base_dir = match self.path.rfind('/') {
                Some(idx) => &self.path[..idx],
                None => "",
            };
            format!("{}/{}", base_dir, url)
        };

        let mut segments = Vec::new();
        for segment in resolved_path.split('/') {
            match segment {
                "" | "." => continue,
                ".." => { segments.pop(); },
                s => segments.push(s),
            }
        }
        
        let normalised_path = format!("/{}", segments.join("/"));

        Url::new(&format!("{}://{}{}", self.scheme, self.host, normalised_path))
    }
}

trait ReadWrite: Read + Write + Debug {}
impl<T: Read + Write + Debug> ReadWrite for T {}
