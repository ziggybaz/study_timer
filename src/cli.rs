use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, PartialEq)]
pub enum Commands {
    Init,
    Add {
        subject: String,

        #[arg(short, long)]
        target_hours: f32,
    },
    Schedule {
        subject: String,
        day:String,
        start_time: String,

        #[arg(short, long)]
        duration: u32,
    },
    List,
    Start,
    Stop,
    Progress,
}
