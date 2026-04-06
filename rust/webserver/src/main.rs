use std::{io::Write, net::{TcpListener, TcpStream}};

fn main() -> std::io::Result<()>{
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    for stream in listener.incoming(){
        handle_request(&mut stream?);
    }
    Ok(())
}

fn handle_request(stream: &mut TcpStream){
    //TODO: use request in response
    let response = "<html><body>Hello World!</body></html>\n".as_bytes();
    let len = response.len();
    let _ = stream.write("HTTP/1.1 200 OK\n".as_bytes());
    let _ = stream.write("content-type: text/html\n".as_bytes());
    let _ = stream.write(format!("content-length: {len}\n").as_bytes());
    let _ = stream.write("\n".as_bytes());
    let _ = stream.write(response);
    let _ = stream.flush();
}
