use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    pub master_password: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(name = "create_user")]
    CreateUser {
        #[arg(short, long)]
        email: String,

        #[arg(short, long)]
        username: String,
    },
}
