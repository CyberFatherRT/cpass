mod cli;
mod proto;

use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = cli::Args::parse();

    println!("master_password = {:?}", cli.master_password);

    match cli.command {
        cli::Commands::CreateUser { email, username } => {
            println!("email = {:?}", email);
            println!("username = {:?}", username);
        }
    }

    Ok(())
}
