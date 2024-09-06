use std::net::TcpListener;
use std::path::PathBuf;
use crate::server::handle_client::handle_client;

pub fn serve(socket_addr: &str, root: PathBuf) -> std::io::Result<()> {
    let listener = TcpListener::bind(socket_addr)?;
    println!("Serving files from {} on {}", root.display(), socket_addr);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let root = root.clone();
                std::thread::spawn(move || {
                    if let Err(e) = handle_client(stream, root) {
                        eprintln!("Error handling client: {:?}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {:?}", e);
            }
        }
    }
    Ok(())
}
