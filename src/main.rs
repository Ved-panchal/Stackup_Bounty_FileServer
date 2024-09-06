mod server;
mod utils;

use std::env;
use std::path::PathBuf;

fn main() -> std::io::Result<()> {
    let root = env::current_dir()?;
    let socket_addr = "localhost:5500";

    server::serve(socket_addr, root)?;
    Ok(())
}
