mod pools;

use clap::Parser;
use eyre::Result;

pub const BUILD_VERSION: &str = version::build_version!();

#[derive(clap::Parser)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, clap::Parser)]
#[command(about = "Common configuration")]
pub struct HttpConfig {
    #[arg(long, env = "SUI_RPC_URL", default_value = "http://127.0.0.1:8545")]
    pub rpc_url: String,
    #[arg(long, help = "deprecated")]
    pub ipc_url: String,
}

#[derive(clap::Subcommand)]
pub struct Command {
    Pools(),
}

fn main() {
    println!("Hello, world! {}", BUILD_VERSION);
}
