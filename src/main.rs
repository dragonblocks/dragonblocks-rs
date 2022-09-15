#![feature(if_let_guard)]

mod client;
mod pkt;
mod quit;
mod server;

use clap::{Parser, Subcommand};
use client::Client;
use quit::Quit;
use server::Server;
use tokio::{signal, task};

#[derive(Parser)]
#[clap(version, about, long_about = None)]
struct Args {
    /// Select client/server/multiplayer mode
    #[clap(subcommand)]
    mode: Mode,
}

#[derive(Subcommand)]
enum Mode {
    /// Connect to a remote server
    Client {
        /// Address to connect to
        #[clap(value_parser)]
        address: String,
    },
    /// Host a server
    Server {
        /// Address to listen to
        #[clap(value_parser)]
        address: String,
    },
    /// Host a local server and connect to it
    Singleplayer {
        /// Optionally open the game to LAN
        #[clap(value_parser, default_value_t = String::from("127.0.0.1:49120"))]
        address: String,
    },
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    env_logger::init();

    let args = Args::parse();

    let (quit, mut quit_confirm) = Quit::new();

    let quit_ctrl_c = quit.clone();
    task::spawn(async move {
        signal::ctrl_c().await.unwrap();
        quit_ctrl_c.quit();
    });

    match args.mode {
        Mode::Server { address } => Server::new(&address, quit).await.run().await,
        Mode::Client { address } => Client::run(&address, quit).await,
        Mode::Singleplayer { address } => {
            let server = Server::new(&address, quit.clone()).await;
            tokio::join!(server.run(), Client::run(&address, quit));
        }
    }

    let _ = quit_confirm.recv().await;
}
