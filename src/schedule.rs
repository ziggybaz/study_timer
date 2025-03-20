use chrono::{DateTime, Duration, Local, NaiveTime};
use std::error::Error;

pub struct Schedule {
    current_session: Option<StudySession>,
}

pub struct StudySession {
    pub subject: String,
    pub start_time: DateTime<Local>,
    pub end_time: DateTime<Local>,
}

impl Schedule {
    pub fn new() -> Self {
        Self {
            current_session: None,
        }
    }

    pub fn start_session(&mut self, subject: &str, duration_minutes: u32) -> Result<(), Box<dyn Error>> {
        let now = Local::now();
        let end_time = now + Duration::minutes(duration_minutes as i64);

        self.current_session = Some(StudySession {
            subject: subject.to_string(),
            start_time: now,
            end_time,
        });

        Ok(())
    }

    pub fn end_session(&mut self) -> Option<StudySession> {
        self.current_session.take()
    }

    pub fn get_current_session(&self) -> Option<&StudySession> {
        self.current_session.as_ref()
    }

    pub fn time_remaining(&self) -> Option<Duration> {
        self.current_session.as_ref().map(|session| {
            let now = Local::now();
            if now < session.end_time {
                session.end_time - now
            } else {
                Duration::zero()
            }
        })
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration as StdDuration;

    #[test]
    fn test_new_schedule() {
        let schedule = Schedule::new();
        assert!(schedule.current_session.is_none());
    }

    #[test]
    fn test_start_session() {
        let mut schedule = Schedule::new();
        let result = schedule.start_session("Rs", 60);
        assert!(result.is_ok());

        let session = schedule.get_current_session().unwrap();
        assert_eq!(session.subject, "Rs");

        let now = Local::now();
        let time_difference = session.end_time.signed_duration_since(now);

        let minutes_difference = time_difference.num_minutes();
        assert!(minutes_difference >= 59 && minutes_difference <= 60);
    }

    #[test]
    fn test_end_session() {
        let mut schedule = Schedule::new();

        schedule.start_session("Rs", 60).unwrap();
        assert!(schedule.get_current_session().is_some());

        let ended_session = schedule.end_session();
        assert!(ended_session.is_some());
        assert_eq!(ended_session.unwrap().subject, "Rs");

        assert!(schedule.get_current_session().is_none());

        let none_session = schedule.end_session();
        assert!(none_session.is_none());
    }

    #[test]
    fn test_get_current_session() {
        let mut schedule = Schedule::new();
        assert!(schedule.get_current_session().is_none());

        schedule.start_session("Rs", 60).unwrap();

        let session = schedule.get_current_session().unwrap();
        assert_eq!(session.subject, "Rs");

        schedule.end_session();

        assert!(schedule.get_current_session().is_none());
    }

    #[test]
    fn test_time_remaining() {
        let mut schedule = Schedule::new();
        assert!(schedule.time_remaining().is_none());

        schedule.start_session("Rs", 3 * 60 / 60).unwrap();

        let time = schedule.time_remaining().unwrap();
        let seconds = time.num_seconds();
        assert!(seconds > 0 && seconds <= 3);

        sleep(StdDuration::from_secs(1));

        let time = schedule.time_remaining().unwrap();
        let seconds = time.num_seconds();
        assert!(seconds > 0 && seconds <= 2);

        sleep(StdDuration::from_secs(3));

        let time = schedule.time_remaining().unwrap();
        assert_eq!(time.num_seconds(), 0);
    }

    #[test]
    fn test_multiple_sessions() {
        let mut schedule = Schedule::new();

        schedule.start_session("Rs", 0).unwrap();
        let first_session = schedule.get_current_session().unwrap();
        assert_eq!(first_session.subject, "Rs");

        shedule.start_session("py", 30).unwrap();
        let second_session = schedule.get_current_session().unwrap();
        assert_eq!(second_session.subject, "py");

        let ended = schedule.end_session().unwrap();
        assert_eq!(ended.subject, "py");

        assert!(schedule.get_current_session().is_none());
    }

    #[test]
    fn test_session_time_boundaries() {
        let mut schedule = Schedule::new();

        schedule.start_session("Rs", 60).unwrap();
        let session = schedule.get_current_session().unwrap();

        let now = Local::now();
        assert!(session.start_time <= now);
        assert!(session.end_time > now);

        let duration = session.end_time.signed_duration_since(session.start_time);
        assert!(duration.num_minutes() >= 59 && duration.num_minutes() <= 60);
    }
}
