mod config;
mod notification;
mod schedule;
mod scheduler;
mod cli;

use clap::Parser;
use cli::{ Cli, Commands };
use scheduler::Scheduler;
use std::process;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let mut scheduler = match Scheduler::new() {
        Ok(scheduler) => scheduler,
        Err(e) => {
            eprintln!("Failed to initialize scheduler: {}", e);
            if cli.command == Commands::Init {
                let scheduler = Scheduler::init()?;
                println!("Conf initialized");
                scheduler
            } else {
                eprintln!("run 'study_timer init' to create initial configuration");
                process::exit(1);
            }
        }
    };

    match cli.command {
        Commands::Init => {},
        Commands::Add { subject, target_hours } => {
            scheduler.add_subject(&subject, target_hours)?;
            println!("Added subject '{}' with a target of {} hours", subject, target_hours);
        },
        Commands::Schedule { subject, day, start_time, duration } => {
            scheduler.add_schedule(&subject, &day, &start_time, duration)?;
            println!("scheduled '{}' on {} at {} for {} minutes", subject, day, start_time, duration);
        },
        Commands::List => {
            scheduler.list_subjects();
        },
        Commands::Start => {
            println!("starting study timer daemon...");
            scheduler.run_daemon().await?;
        },
        Commands::Stop => {
            println!("stopping study ttimer daemon...");
            scheduler.stop_daemon()?;
        },
        Commands::Progress => {
            scheduler.show_progress();
        },
    }

    Ok(())
}
