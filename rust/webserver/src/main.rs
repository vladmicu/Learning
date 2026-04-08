use core::fmt;
use std::{collections::HashMap, io::{Read, Write}, net::{TcpListener, TcpStream}};

fn main() -> std::io::Result<()>{
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    for stream in listener.incoming(){
        handle_request(&mut stream?);
    }
    Ok(())
}

fn handle_request(stream: &mut TcpStream){
    let mut raw_request_bytes : Vec<u8> = vec![];
    let mut buff: [u8; 1000] = [0;1000];
    let mut read_result = stream.read(&mut buff);
    while let Ok(n) = read_result{
        raw_request_bytes.extend_from_slice(&buff[0..n]);
        if n < buff.len() {
            break;
        }
        //buff = [0; 1000]; //doesn't seem necessary, tested with smaller buffer size that requires multiple iterations to read entire request 
        read_result = stream.read(&mut buff);
    }
    let Ok(_) = read_result else{
        return error_response(stream, HttpError::InternalServerError);
    };
    let Ok(raw_request) = str::from_utf8(&raw_request_bytes) else{
        return error_response(stream, HttpError::BadRequest);
    };
    //println!("Request:");
    //println!("{raw_request}");
    let req = Request::parse(raw_request);
    match req{
        Err(err) => error_response(stream, err),
        Ok(r) => normal_response(stream, r)
    }
}

fn error_response(stream: &mut TcpStream, err: HttpError){
    let _ = stream.write(format!("HTTP/1.1 {err}").as_bytes());
    let _ = stream.flush();
}

fn normal_response(stream: &mut TcpStream, req : Request){
    if req.resource != "/" {
        return error_response(stream, HttpError::NotFound);
    }
    if req.method != HttpMethod::GET {
        return error_response(stream, HttpError::MethodNotAllowed);
    }

    let name =  req.headers.get("name")
                            .or(req.params.get("name"))
                            .unwrap_or(&"World");
    let str_content = format!("<html><body>Hello {name}!</body></html>\n");
    let content = str_content.as_bytes();
    let content_length = content.len();
    let _ = stream.write("HTTP/1.1 200 OK\n".as_bytes());
    let _ = stream.write("content-type: text/html\n".as_bytes());
    let _ = stream.write(format!("content-length: {content_length}\n").as_bytes());
    let _ = stream.write("\n".as_bytes());
    let _ = stream.write(content);
    let _ = stream.flush();
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