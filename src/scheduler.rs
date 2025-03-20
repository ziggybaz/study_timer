use crate::config::{Config, Subject, StudySession};
use crate::notification::Notifier;
use chrono::{DateTime, Datelike, Local, NaiveTime, Timelike, Weekday};
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::time::Duration;
use tokio::{task, time};
use colored::Colorize;

pub struct Scheduler {
    config: Config,
    notifier: Notifier,
    running: Arc<AtomicBool>,
}

impl Scheduler {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let config = Config::load()?;
        let notifier = Notifier::new();

        Ok(Self {
            config,
            notifier,
            running: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn init() -> Result<Self, Box<dyn Error>> {
        let config = Config::default();
        config.save()?;
        let notifier = Notifier::new();

        Ok(Self {
            config,
            notifier,
            running: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn add_subject(&mut self, name: &str, target_hours: f32) -> Result<(), Box<dyn Error>> {
        self.config.add_subject(name, target_hours)?;
        self.config.save()?;
        Ok(())
    }

    pub fn add_schedule(&mut self, subject: &str, day: &str, start_time: &str, duration: u32) -> Result<(), Box<dyn Error>> {
        self.config.add_schedule(subject, day, start_time, duration)?;
        self.config.save()?;
        Ok(())
    }

    pub fn list_subjects(&self) {
        println!("{}", "Subjects and schedules:".bold());
        println!("{}", "-".repeat(50));

        for (name, subject) in &self.config.subjects {
            println!("{}: {} hours target", name.green().bold(), subject.target_hours);
            println!(" Progress: {:.1}/{:.1} hours ({:.1}%)",
            subject.completed_hours,
            subject.target_hours,
            (subject.completed_hours / subject.target_hours) * 100.0);

            if let Some(sessions) = self.config.schedules.get(name) {
                println!(" Scheduled sessions:");
                for session in sessions {
                    println!("  {} at {} for {} minutes",
                             session.day.blue(),
                             session.start_time,
                             session.duration);
                }
            } else {
                println!("  No scheduled sessions");
            }
            println!();
        }
    }

    pub async fn run_daemon(&mut self) -> Result<(), Box<dyn Error>> {
        self.running.store(true, Ordering::SeqCst);
        let running = Arc::clone(&self.running);

        let schedules = self.config.schedules.clone();
        let subjects = self.config.subjects.clone();
        let config_path = self.config.config_path.clone();

        let notifier = self.notifier.clone();

        task::spawn(async move {
            println!("study timer daemon started");

            while running.load(Ordering::SeqCst) {
                let now = Local::now();
                let current_day = match now.weekday() {
                    Weekday::Mon => "Monday",
                    Weekday::Tue => "Tuesday",
                    Weekday::Wed => "Wednesday",
                    Weekday::Thu => "Thursday",
                    Weekday::Fri => "Friday",
                    Weekday::Sat => "Saturday",
                    Weekday::Sun => "Sunday",
                };

                let current_time = now.format("%H:%M").to_string();

                for (subject_name, sessions) in &schedules {
                    for session in sessions {
                        if session.day == current_day && session.start_time == current_time {
                            let message = format!("Time to study {} for {} minutes", subject_name, session.duration);
                            notifier.notify("Study Timer", &message);
                        }
                        if session.day == current_day {
                            if let Ok(session_time) = NaiveTime::parse_from_str(&session.start_time, "%H:%M") {
                                let now_time = NaiveTime::from_hms_opt(now.hour(), now.minute(), 0).unwrap();
                                let diff_minutes = (session_time.signed_duration_since(now_time).num_minutes() + 60) %60;

                                if diff_minutes == 5 {
                                    let message = format!("{} study session starts in 5 minutes", subject_name);
                                    notifier.notify("study timer", &message);
                                }
                            }
                        }
                    }
                }

                time::sleep(Duration::from_secs(60)).await;
            }

            println!("study timer daemon stopped");
        });

        Ok(())
    }

    pub fn stop_daemon(&self) -> Result<(), Box<dyn Error>> {
        self.running.store(false, Ordering::SeqCst);
        println!("sent stop signal to daemon");
        Ok(())
    }

    pub fn show_progress(&self) {
        println!("{}", "study progress:".bold());
        println!("{}", "-".repeat(50));

        let mut total_completed = 0.0;
        let mut total_target = 0.0;

        for (name, subject) in &self.config.subjects {
            total_completed += subject.completed_hours;
            total_target += subject.target_hours;

            let percentage = (subject.completed_hours / subject.target_hours) * 100.0;
            let progress_bar = self.generate_progress_bar(percentage);

            println!("{}: {:.1}/{:.1} hours", name.green().bold(), subject.completed_hours, subject.target_hours);
            println!("{} {:.1}%", progress_bar, percentage);
        }

        println!("\n{}", "Overall progress:".bold());
        let overall_percentage = (total_completed / total_target) * 100.0;
        let overall_bar = self.generate_progress_bar(overall_percentage);
        println!("{} {:.1}%", overall_bar, overall_percentage);
    }

    fn generate_progress_bar(&self, percentage: f32) -> String {
        let width = 30;
        let filled = (percentage / 100.0 * width as f32).round() as usize;
        let empty = width - filled;

        format!("[{}{}]", "█".repeat(filled).green(), "░".repeat(empty))
    }
}
