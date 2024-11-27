use alloc::string::String;

/// Struct representing a URL. Only HTTP scheme is supported.
#[derive(Debug, Clone, PartialEq)]
pub struct Url {
    url: String,
    host: String,
    port: String,
    path: String,
    search_part: String,
}

use alloc::string::ToString;
use alloc::vec::Vec;
impl Url {
    pub fn new(url: String) -> Self {
        Self {
            url,
            host: "".to_string(),
            port: "".to_string(),
            path: "".to_string(),
            search_part: "".to_string(),
        }
    }
    pub fn parse(&mut self) -> Result<Self, String> {
        if !self.is_http() {
            return Err("Only HTTP scheme is supported.".to_string());
        }
        self.host = self.extract_host();
        self.port = self.extract_port();
        self.path = self.extract_path();
        self.search_part = self.extract_search_part();
        Ok(self.clone())
    }
    fn is_http(&mut self) -> bool {
        if self.url.contains("http://") {
            return true;
        }
        false
    }
    fn extract_host(&self) -> String {
        let url_parts: Vec<&str> = self
            .url
            .trim_start_matches("http://")
            .splitn(2, "/")
            .collect();
        if let Some(index) = url_parts[0].find(':') {
            url_parts[0][..index].to_string()
        } else {
            url_parts[0].to_string()
        }
    }
    fn extract_port(&self) -> String {
        let url_parts: Vec<&str> = self
            .url
            .trim_start_matches("http://")
            .splitn(2, "/")
            .collect();
        if let Some(index) = url_parts[0].find(':') {
            url_parts[0][index + 1..].to_string()
        } else {
            "80".to_string()
        }
    }
    fn extract_path(&self) -> String {
        let url_parts: Vec<&str> = self
            .url
            .trim_start_matches("http://")
            .splitn(2, "/")
            .collect();
        if url_parts.len() < 2 {
            return "".to_string();
        }
        let path_and_search_part: Vec<&str> = url_parts[1].splitn(2, "?").collect();
        path_and_search_part[0].to_string()
    }
    fn extract_search_part(&self) -> String {
        let url_parts: Vec<&str> = self
            .url
            .trim_start_matches("http://")
            .splitn(2, "/")
            .collect();
        if url_parts.len() < 2 {
            return "".to_string();
        }
        let path_and_search_part: Vec<&str> = url_parts[1].splitn(2, "?").collect();
        if path_and_search_part.len() < 2 {
            "".to_string()
        } else {
            path_and_search_part[1].to_string()
        }
    }
    pub fn host(&self) -> String {
        self.host.clone()
    }
    pub fn port(&self) -> String {
        self.port.clone()
    }
    pub fn path(&self) -> String {
        self.path.clone()
    }
    pub fn search_part(&self) -> String {
        self.search_part.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_url_host() {
        let url = "http://example.com".to_string();
        let expected = Ok(Url {
            url: url.clone(),
            host: "example.com".to_string(),
            port: "80".to_string(),
            path: "".to_string(),
            search_part: "".to_string(),
        });
        assert_eq!(expected, Url::new(url).parse());
    }
    #[test]
    fn test_url_port() {
        let url = "http://example.com:8888".to_string();
        let expected = Ok(Url {
            url: url.clone(),
            host: "example.com".to_string(),
            port: "8888".to_string(),
            path: "".to_string(),
            search_part: "".to_string(),
        });
        assert_eq!(expected, Url::new(url).parse());
    }
    #[test]
    fn test_url_port_path() {
        let url = "http://example.com:8888/index.html".to_string();
        let expected = Ok(Url {
            url: url.clone(),
            host: "example.com".to_string(),
            port: "8888".to_string(),
            path: "index.html".to_string(),
            search_part: "".to_string(),
        });
        assert_eq!(expected, Url::new(url).parse());
    }
    #[test]
    fn test_url_host_path() {
        let url = "http://example.com/index.html".to_string();
        let expected = Ok(Url {
            url: url.clone(),
            host: "example.com".to_string(),
            port: "80".to_string(),
            path: "index.html".to_string(),
            search_part: "".to_string(),
        });
        assert_eq!(expected, Url::new(url).parse());
    }
    #[test]
    fn test_url_port_path_search_query() {
        let url = "http://example.com:8888/index.html?a=123&b=456".to_string();
        let expected = Ok(Url {
            url: url.clone(),
            host: "example.com".to_string(),
            port: "8888".to_string(),
            path: "index.html".to_string(),
            search_part: "a=123&b=456".to_string(),
        });
        assert_eq!(expected, Url::new(url).parse());
    }
    #[test]
    fn test_no_scheme() {
        let url = "example.com".to_string();
        let expected = Err("Only HTTP scheme is supported.".to_string());
        assert_eq!(expected, Url::new(url).parse());
    }
    #[test]
    fn test_unsupported_scheme() {
        let url = "https://example.com:8888/index.html".to_string();
        let expected = Err("Only HTTP scheme is supported.".to_string());
        assert_eq!(expected, Url::new(url).parse());
    }
}
