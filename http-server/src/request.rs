use std::{io::Read, net::TcpStream};

#[derive(Debug)]
#[derive(PartialEq)]
pub struct Request {
    method: String,
    path: String,
    version: String,
    headers: Option<Vec<String>>,
    body: Option<String>,
}

impl Request {
    pub fn new(
        method: &str,
        path: &str,
        version: &str,
        headers: Option<Vec<String>>,
        body: Option<String>,
    ) -> Request {
        Request { 
        method: method.to_string(),
        path: path.to_string(),
        version: version.to_string(),
        headers,
        body,
        }
    }
    pub fn read_from(stream: &mut TcpStream) -> Result<Request, Box<dyn std::error::Error>> {
        let mut buffer = [0; 1024];

        if stream.read(&mut buffer).is_err() {
            stream.shutdown(std::net::Shutdown::Both)?;
    }
        println!("Read from socket: {}", String::from_utf8_lossy(&buffer[..]));

        let request = Request::from_string(std::str::from_utf8(&buffer)?);
        println!("Request: {:?}", request);
        return request;
    }

    pub fn from_string(request_string: &str) -> Result<Request, Box<dyn std::error::Error>> {
        
        let lines: Vec<&str> = request_string.lines().collect();
        println!("{:?}", lines);
        let start_line = lines[0];
        let start_line_elements: Vec<&str> = start_line.split(' ').collect();
        if start_line_elements.len() != 3 {
            return Err("Bad request".into());
        }
        let method = start_line_elements[0];
        let path = start_line_elements[1];
        let version = start_line_elements[2];
        let mut headers = None;
        let mut body = None;

        //si tiene headers
        if lines.len() > 1 {
            let mut headers_vec = vec![];

            //si tiene body
            if lines[lines.len()-2] == "" {
                let headers_strings = &lines[1..lines.len()-2];
                for header in headers_strings {
                    let header = String::from(*header);
                    headers_vec.push(header);
                }
                body = Some(lines[lines.len()-1].to_string());
            } else {
                for header in &lines[1..] {
                    let header = String::from(*header);
                    headers_vec.push(header);
                }
            }
            headers = Some(headers_vec);
        }

        return Ok(Request {
            method: method.to_string(),
            path: path.to_string(),
            version: version.to_string(),
            headers,
            body: body,
        });
    }

    // "simple get" -> el tipo de request que haría un browser para conectarse a este server
    pub fn is_simple_get(&self) -> bool {
        self.method == "GET".to_owned() && 
        self.path == "/".to_owned() &&
        self.version == "HTTP/1.1".to_owned()       
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test01_valid_request() {
        let request_string = "GET / HTTP/1.1\r\nContent-Length: 4\r\n\r\nasdf";
        let expected_request = Request::new(
            "GET", 
            "/", 
            "HTTP/1.1", 
            Some(vec!["Content-Length: 4".to_string()]),
            Some("asdf".to_string()),
        );
        assert_eq!(Request::from_string(request_string).unwrap(), expected_request);
    }

    #[test]
    fn test02_valid_request_with_no_body() {
        let request_string = "GET / HTTP/1.1\r\nContent-Length: 4\r\n";
        let expected_request = Request::new(
            "GET", 
            "/", 
            "HTTP/1.1", 
            Some(vec!["Content-Length: 4".to_string()]),
            None,
        );
        assert_eq!(Request::from_string(request_string).unwrap(), expected_request);
    }

    #[test]
    fn test03_valid_request_with_no_headers() {
        let request_string = "GET / HTTP/1.1\r\n";
        let expected_request = Request::new(
            "GET", 
            "/", 
            "HTTP/1.1", 
            None,
            None,
        );
        assert_eq!(Request::from_string(request_string).unwrap(), expected_request);
    }

    #[test]
    fn test04_invalid_request() {
        let request_string = "GET/ HTTP/1.1\r\n";
        assert!(Request::from_string(request_string).is_err());
    }

    #[test]
    fn test05_valid_request_with_multiple_headers() {
        let request_string = "GET / HTTP/1.1\r\nHost: localhost:8000\r\nContent-Length: 4\r\n\r\nasdf";
        let expected_request = Request::new(
            "GET", 
            "/", 
            "HTTP/1.1", 
            Some(vec!["Host: localhost:8000".to_string(), "Content-Length: 4".to_string()]),
            Some("asdf".to_string()),
        );
        assert_eq!(Request::from_string(request_string).unwrap(), expected_request);
    }
}