pub enum HTTPMethod {
  GET,
  HEAD,
  POST,
  PUT,
  DELETE,
  CONNECT,
  OPTIONS,
  TRACE,
  PATCH,
}

pub struct Request {
  method: HTTPMethod,
  path: String,
  version: String,
  headers: Option<Vec<String>>,
  body: Option<String>,
}

impl Request {
  
}