use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::error::Error;
use directories::ProjectDirs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Subject {
    pub target_hours: f32,
    pub completed_hours: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StudySession {
    pub day: String,
    pub start_time: String,
    pub duration: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub subjects: HashMap<String, Subject>,
    pub schedules: HashMap<String, Vec<StudySession>>,
    pub config_path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        let config_path = Self::get_config_path();
        Self {
            subjects: HashMap::new(),
            schedules: HashMap::new(),
            config_path,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn Error>> {
        let config_path = Self::get_config_path();

        if !config_path.exists() {
            return Err("config file not found".into());
        }

        let config_str = fs::read_to_string(&config_path)?;
        let mut config: Config = serde_json::from_str(&config_str)?;
        config.config_path = config_path;

        Ok(config)
    }

    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let config_str = serde_json::to_string_pretty(self)?;
        let mut file = File::create(&self.config_path)?;
        file.write_all(config_str.as_bytes())?;

        Ok(())
    }

    pub fn add_subject(&mut self, name: &str, target_hours: f32) -> Result<(), Box<dyn Error>> {
        if target_hours <= 0.0 {
            return Err("you must set a target time for yoyr study".into());
        }

        self.subjects.insert(name.to_string(), Subject {
            target_hours,
            completed_hours: 0.0,
        });

        Ok(())
    }

    pub fn add_schedule(&mut self, subject: &str, day: &str, start_time: &str, duration: u32) -> Result<(), Box<dyn Error>> {
        if !self.subjects.contains_key(subject) {
            return Err(format!("subject '{}' not found..", subject).into());
        }

        let valid_days = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
        if !valid_days.contains(&day) {
            return Err(format!("incorrect day '{}', must be one of: {}", day, valid_days.join(" ")).into());
        }

        if !start_time.matches(|c| c == ':').count() == 1 {
            return Err("Time must be in 'HH:MM' format".into());
        }

        let session = StudySession {
            day: day.to_string(),
            start_time: start_time.to_string(),
            duration,
        };

        self.schedules
            .entry(subject.to_string())
            .or_insert_with(Vec::new)
            .push(session);

        Ok(())
    }

    fn get_config_path() -> PathBuf {
        if let Some(project_directories) = ProjectDirs::from("com", "study_timer", "study_timer") {
            project_directories.config_dir().join("config.json")
        } else {
            PathBuf::from("./study_timer_config.json")
        }
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    use std::io::Read;
    use tempfile::tempdir;

    fn create_test_config() -> Config {
        let temp_dir = tempdir().expect("failed tp create temp directory");
        let config_path = temp_dir.path().join("test_config.json");

        let mut config = Config::default();
        config.config_path = config_path;

        config
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();

        assert!(config.subjects.is_empty());
        assert!(config.schedules.is_empty());
        assert!(!config.config_path.as_os_str().is_empty());
    }

    #[test]
    fn test_add_subject() {
        let mut config = create_test_config();
        let result = config.add_subject("dsa", 10.0);

        assert!(result.is_ok());
        assert!(config.subjects.contains_key("dsa"));
        assert_eq!(config.subjects.get("dsa").unwrap().target_hours, 10.0);
        assert_eq!(config.subjects.get("dsa").unwrap().completed_hours, 0.0);

        let result = config.add_subject("ml/ai", 0.0);
        assert!(result.is_err());
        assert!(!config.subjects.contains_key("ml/ai"));

        let result = config.add_subject("poetry", -5.0);
        assert!(result.is_err());
        assert!(!config.subjects.contains_key("poetry"));
    }

    #[test]
    fn test_add_schedule() {
        let mut config = create_test_config();

        config.add_subject("QA", 10.0).unwrap();

        let result = config.add_schedule("QA", "Monday", "09:00", 60);
        assert!(result.is_ok());

        let qa_schedules = config.schedules.get("QA").unwrap();
        assert_eq!(qa_schedules.len(), 1);
        assert_eq!(qa_schedules[0].day, "Monday");
        assert_eq!(qa_schedules[0].start_time, "09:00");
        assert_eq!(qa_schedules[0].duration, 60);

        let result = config.add_schedule("embedded", "Monday", "10:00", 30);
        assert!(result.is_err());
        assert!(!config.schedules.contains_key("embedded"));

        let result = config.add_schedule("BE", "ijumaa", "10:00", 30);
        assert!(result.is_err());

        let qa_schedules = config.schedules.get("QA").unwrap();
        assert_eq!(qa_schedules.len(), 1);

        config.add_schedule("QA", "Tuesday", "14:00", 45).unwrap();

        let qa_schedules = config.schedules.get("QA").unwrap();
        assert_eq!(qa_schedules.len(), 2);
        assert_eq!(qa_schedules[1].day, "Tuesday");
    }

    #[test]
    fn test_save_and_load() {
        let mut config = create_test_config();

        config.add_subject("DB", 10.0).unwrap();
        config.add_schedule("DB", "Monday", "09:00", 50).unwrap();

        let save_result = config.save();
        assert!(save_result.is_ok());
        assert!(config.config_path.exists());

        let mut file = File::open(&config.config_path).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();

        assert!(content.contains("DB"));
        assert!(content.contains("Monday"));
        assert!(content.contains("09:00"));

        let config_path = config.config_path.clone();

        let mut new_config = Config::default();
        new_config.config_path = config_path;
        new_config.save().unwrap();

        let loaded_config = Config::load().unwrap();

        assert!(loaded_config.subjects.contains_key("DB"));
        assert_eq!(loaded_config.subjects.get("DB").unwrap().target_hours, 10.0);
        assert!(loaded_config.schedules.contains_key("DB"));

        let loaded_schedules = loaded_config.schedules.get("DB").unwrap();
        assert_eq!(loaded_schedules[0].day, "Monday");
        assert_eq!(loaded_schedules[0].start_time, "09:00");
    }

    #[test]
    fn test_load_nonexistent_config() {
        let temp_dir = tempdir().expect("failed to create temporary directory");
        let non_existent_path = temp_dir.path().join("404.json");

        let mut config = Config::default();
        config.config_path = non_existent_path;

        let result = Config::load();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_time_format() {
        let mut config = create_test_config();
        config.add_subject("OS", 10.0).unwrap();

        let result = config.add_schedule("OS", "Wednesday", "1000", 60);
        assert!(result.is_err());

        let result = config.add_schedule("OS", "Wednesday", "10:00:00", 60);
        assert!(result.is_err());
    }
}
