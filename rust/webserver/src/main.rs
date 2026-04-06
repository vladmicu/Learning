use std::net::TcpListener;

fn main() -> std::io::Result<()>{
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    for _stream in listener.incoming(){
        println!("Recieved something"); //TODO: handle request 
    }
    Ok(())
}
