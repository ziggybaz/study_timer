use notify_rust::{Notification, NotificationHandle};
use std::error::Error;

#[derive(Clone)]
pub struct Notifier {
} //TODO:I'll add configuration options later

impl Notifier {
    pub fn new() -> Self {
        Self {}
    }

    pub fn notify(&self, title: &str, message: &str) -> Result<NotificationHandle, Box<dyn Error>> {
        let notification = Notification::new()
            .summary(title)
            .body(message)
            .icon("clock")
            .timeout(10000)
            .show()?;

        Ok(notification)
    }
}
