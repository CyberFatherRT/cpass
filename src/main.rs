mod password_store;

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about=None)]
struct Args {
    #[arg(short, long)]
    name: String,

    #[arg(short, long)]
    password: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    Ok(())
}
