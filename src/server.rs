use crate::options::Options;
use anyhow::Result;
use std::net::SocketAddr;
use tokio::net::TcpListener;

pub struct Server {
    addr: SocketAddr,
}

impl Server {
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }

    pub async fn run(&self) -> Result<()> {
        let listener = TcpListener::bind(self.addr).await?;
        println!("Server listening on {}", self.addr);

        loop {
            let (socket, addr) = listener.accept().await?;
            println!("Connection from {}", addr);

            // Handle connection
            tokio::spawn(async move {
                let _ = socket;
            });
        }
    }
}

pub async fn start_server(options: &Options) -> Result<()> {
    let addr = if let Some(ref listen) = options.listen {
        listen.parse()?
    } else {
        "127.0.0.1:6266".parse()?
    };

    let server = Server::new(addr);
    server.run().await
}
