use core::fmt;
use std::{collections::HashMap, io::{BufRead, BufReader, Write}, net::{TcpListener, TcpStream}};

type ResponseFunction = fn(Request) -> Result<Vec<u8>, HttpError>; 

fn main() -> std::io::Result<()>{
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    let mut response_map : HashMap<&str, ResponseFunction> = HashMap::new();
    response_map.insert("/", get_response_index); 
    for stream in listener.incoming(){
        handle_request(&mut stream?, &response_map);
    }
    Ok(())
}

fn handle_request(stream: &mut TcpStream, response_map: &HashMap<&str, ResponseFunction>){
    let raw_request = match read_request(stream){
        Err(err) => return error_response(stream, err),
        Ok(s) => s
    };
    //println!("Request:");
    //println!("{raw_request}");
    let req = Request::parse(raw_request.as_str());
    match req{
        Err(err) => error_response(stream, err),
        Ok(r) => normal_response(stream, r, &response_map)
    }
}

fn read_request(stream: &mut TcpStream) -> Result<String, HttpError>{
    let mut reader = BufReader::new(stream);
    let buffer = reader.fill_buf().map_err(|_| HttpError::InternalServerError)?;
    let length = buffer.len();
    let buffer_str = str::from_utf8(buffer).map_err(|_| HttpError::BadRequest)?;
    let raw_request = buffer_str.to_owned();
    reader.consume(length);
    Ok(raw_request)
}

fn error_response(stream: &mut TcpStream, err: HttpError){
    let _ = stream.write(format!("HTTP/1.1 {err}").as_bytes());
    let _ = stream.flush();
}

fn normal_response(stream: &mut TcpStream, req : Request, response_map: &HashMap<&str, ResponseFunction>){
    let response_function = match response_map.get(req.resource){
        None => return error_response(stream, HttpError::NotFound),
        Some(f) => f
    };

    let response = match response_function(req){
        Err(err) => return error_response(stream, err),
        Ok(r) => r
    };
    let _ = stream.write_all(&response);
    let _ = stream.flush();
}

fn get_response_index(req : Request) -> Result<Vec<u8>,HttpError>{
    if req.method != HttpMethod::GET {
        return Err(HttpError::MethodNotAllowed);
    }
    let name =  req.headers.get("name")
                            .or(req.params.get("name"))
                            .unwrap_or(&"World");
    let str_content = format!("<html><body>Hello {name}!</body></html>\n");
    let content = str_content.as_bytes();
    let content_length = content.len();

    let mut response : Vec<u8> = Vec::new();
    response.extend_from_slice("HTTP/1.1 200 OK\n".as_bytes());
    response.extend_from_slice("content-type: text/html\n".as_bytes());
    response.extend_from_slice(format!("content-length: {content_length}\n").as_bytes());
    response.extend_from_slice("\n".as_bytes());
    response.extend_from_slice(content);
    Ok(response)
}

#[derive(PartialEq)]
enum HttpMethod{
    GET,
    HEAD,
    POST,
    PUT,
    DELETE,
    PATCH,
    OPTIONS,
    CONNECT,
    TRACE
}

fn get_method(name: &str) -> Option<HttpMethod>{
    match name {
        "GET" => Some(HttpMethod::GET),
        "HEAD" => Some(HttpMethod::HEAD),
        "POST" => Some(HttpMethod::POST),
        "PUT" => Some(HttpMethod::PUT),
        "DELETE" => Some(HttpMethod::DELETE),
        "PATCH" => Some(HttpMethod::PATCH),
        "OPTIONS" => Some(HttpMethod::OPTIONS),
        "CONNECT" => Some(HttpMethod::CONNECT),
        "TRACE" => Some(HttpMethod::TRACE),
        _ => None
    }
}

struct Request<'a>{
    method: HttpMethod,
    resource: &'a str,
    headers: HashMap<&'a str, &'a str>,
    params: HashMap<&'a str, &'a str>,
    content: &'a str
}

impl<'a> Request<'a> {
    fn parse(raw_request: &'a str) -> Result<Self, HttpError>{
        let delimiter = raw_request.find("\n\n")
                        .or_else(|| raw_request.find("\r\n\r\n"));
        let (raw_headers, content) = match delimiter{
            None => (raw_request, ""),
            Some(index) => raw_request.split_at(index)
        };
        let mut lines = raw_headers.lines();

        let first_line = lines.nth(0).ok_or(HttpError::BadRequest)?;
        let mut parts = first_line.split_whitespace();
        let raw_method = parts.nth(0).ok_or(HttpError::BadRequest)?;
        let res_and_params = parts.nth(0).ok_or(HttpError::BadRequest)?;
        let method = get_method(raw_method).ok_or(HttpError::NotImplemented)?;
        let param_delimiter = res_and_params.find('?');
        let (resource, raw_parameters) = match param_delimiter {
            None => (res_and_params, ""),
            Some(index) => res_and_params.split_at(index)
        };
        let params : HashMap<&str, &str> = match raw_parameters{
            "" => HashMap::new(),
            _ => raw_parameters.trim_start_matches('?')
                               .split('&')
                               .map(|line| {
                                    line.find('=').map(|index| line.split_at(index))
                                                  .map(|(key, value)| (key, value.trim_start_matches('=')))

                                })
                               .filter(Option::is_some)
                               .map(Option::unwrap)
                               .collect()
        };
        let headers : HashMap<&str, &str> = lines.map(|line|{
                                                        line.find(": ").map(|index| line.split_at(index))
                                                                       .map(|(key, value)| (key, value.trim_start_matches(": ")))
                                                    }) 
                                                    .filter(Option::is_some)
                                                    .map(Option::unwrap)
                                                    .collect();
        let request = Request{
            method: method,
            resource: resource,
            headers: headers,
            params: params,
            content: content
        };
        Ok(request)
    }
}

enum HttpError{
    NotFound,
    NotImplemented,
    MethodNotAllowed,
    BadRequest,
    InternalServerError
}

impl fmt::Display for HttpError{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        let msg = match self {
            HttpError::NotFound => "404 Not Found",
            HttpError::NotImplemented => "501 Not Implemented",
            HttpError::MethodNotAllowed => "405 Method Not Allowed",
            HttpError::BadRequest => "400 Bad Request",
            HttpError::InternalServerError => "500 Internal Server Error"
        };
        write!(f, "{msg}")
    }
}