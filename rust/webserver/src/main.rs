use core::fmt;
use std::{collections::HashMap, io::{BufRead, BufReader, Write}, net::{TcpListener, TcpStream}};

fn main() -> std::io::Result<()>{
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    for stream in listener.incoming(){
        handle_request(&mut stream?);
    }
    Ok(())
}

fn handle_request(stream: &mut TcpStream){
    let mut reader = BufReader::new(stream);
    let req = parse_and_validate(&mut reader);
    let stream = reader.into_inner();
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
    let name = match req.headers.get("name"){
        Some(s) => s,
        None => match req.params.get("name"){
            Some(s) => s,
            None => "World"
        }
    };
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

fn get_method(name: String) -> Option<HttpMethod>{
    match name.as_str() {
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

struct Request{
    method: HttpMethod,
    resource: String,
    headers: HashMap<String, String>,
    params: HashMap<String, String>
}

enum HttpError{
    NotFound,
    NotImplemented,
    MethodNotAllowed,
    BadRequest
}

impl fmt::Display for HttpError{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        let msg = match self {
            HttpError::NotFound => "404 Not Found",
            HttpError::NotImplemented => "501 Not Implemented",
            HttpError::MethodNotAllowed => "405 Method Not Allowed",
            HttpError::BadRequest => "400 Bad Request"
        };
        write!(f, "{msg}")
    }
}

fn parse_and_validate(reader: &mut BufReader<&mut TcpStream>) -> Result<Request, HttpError>{
    let mut first_line = String::new();
    let len = reader.read_line(&mut first_line);
    match len {
        Err(_) => return Err(HttpError::BadRequest),
        Ok(sz) => if sz == 0 {return Err(HttpError::BadRequest);} else {()}
    }
    let parts : Vec<&str> = first_line.split(' ').collect();
    if parts.len() < 2 {
        return Err(HttpError::BadRequest);
    }
    let res_and_params: Vec<&str> = parts[1].split('?').collect();
    let resource = match res_and_params[0]{
        "/" => String::from("/"),
        _ => return Err(HttpError::NotFound)
    };
    let params = match get_params(res_and_params){
        Err(e) => return Err(e),
        Ok(p) => p
    };

    let method = match get_method(String::from(parts[0])){
        None => return Err(HttpError::NotImplemented),
        Some(m) => m
    };
    match method{
        HttpMethod::GET => (),
        _ => return Err(HttpError::MethodNotAllowed)
    };
    let headers = match get_headers(reader) {
        Err(e) => return Err(e),
        Ok(h) => h
    };

    let req = Request{
        method: method,
        resource: resource,
        headers: headers,
        params: params
    };
    Ok(req)
} 

fn get_headers(reader: &mut BufReader<&mut TcpStream>) -> Result<HashMap<String, String>, HttpError>{
    let mut headers: HashMap<String, String> = HashMap::new();
    let mut header = String::new();
    let mut len = reader.read_line(&mut header);
    let mut keep_going: bool = match len {
        Err(_) => false,
        Ok(l) => match l {
            0 => false,
            _ => true
        }
    };
    while keep_going{
        let key_val : Vec<String> = header.split(": ")
                                          .map(String::from)
                                          .collect();
        if key_val.len() != 2 {
            if key_val.len() > 2{
                return Err(HttpError::BadRequest);
            } else {
                break;
            }
        }
        let mut iter = key_val.into_iter();
        headers.insert(iter.next().unwrap(), iter.next().unwrap());

        header = String::new();
        len = reader.read_line(&mut header);
        keep_going = match len {
            Err(_) => false,
            Ok(l) => match l {
                0 => false,
                _ => true
            }
        };
    }
    Ok(headers)
}

fn get_params(res_and_params: Vec<&str>) -> Result<HashMap<String, String>, HttpError>{
    let mut params: HashMap<String, String> = HashMap::new();
    if res_and_params.len() > 1 {
        for param in res_and_params[1].split('&'){
            let key_val: Vec<String> = param.split('=')
                                            .map(String::from)
                                            .collect();
            if key_val.len()  != 2 {
                return Err(HttpError::BadRequest);
            }
            let mut iter = key_val.into_iter();
            params.insert(iter.next().unwrap(), iter.next().unwrap());
        }
    }
    Ok(params)
}
