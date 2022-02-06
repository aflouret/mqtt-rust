use std::{net::TcpStream, io::Write};

pub struct Response {
    version: String,
    status_code: u16,
    status_text: String,
    headers: Option<Vec<String>>,
    body: Option<String>,
}

impl Response {
    pub fn new(
        version: &str,
        status_code: u16,
        status_text: &str,
        headers: Option<Vec<String>>,
        body: Option<String>,
    ) -> Response {
        Response { 
        version: version.to_string(),
        status_code,
        status_text: status_text.to_string(),
        headers,
        body,
        }
    }

  pub fn write_to(&self, stream: &mut TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    stream.write_all(self.to_string().as_bytes())?;
    stream.flush()?;
    Ok(())
  }

  pub fn to_string(&self) -> String {
    let mut response_string = format!(
      "{} {} {}",
      self.version,
      self.status_code,
      self.status_text
    );
    if let Some(headers) = &self.headers {
      for header in headers {
        response_string.push_str("\r\n");
        response_string.push_str(&header);
      }
    }
    if let Some(body) = &self.body {
        response_string.push_str("\r\n\r\n");
        response_string.push_str(body);
    }
    
    return response_string;
  }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test01() {
        let body = "asdf";
        let content_length_header = format!("Content-Length: {}", body.len());
        let response = Response::new(
          "HTTP/1.1",
          200,
          "OK",
          Some(vec![content_length_header.to_string()]),
          Some(body.to_string())
        );
        
        let expected_string = "HTTP/1.1 200 OK\r\nContent-Length: 4\r\n\r\nasdf";
        assert_eq!(response.to_string(), expected_string);

    }
}