// Event timing logic
// Handles cron-like scheduling patterns

use chrono::{DateTime, Datelike, Timelike};
use serde::{Deserialize, Serialize};

/// Event timing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timing {
    #[serde(default)]
    pub years: Option<Vec<u32>>,
    
    #[serde(default)]
    pub months: Option<Vec<u32>>,
    
    #[serde(default)]
    pub days: Option<Vec<u32>>,
    
    #[serde(default)]
    pub weekdays: Option<Vec<u32>>,
    
    #[serde(default)]
    pub hours: Option<Vec<u32>>,
    
    #[serde(default)]
    pub minutes: Option<Vec<u32>>,
}

impl Timing {
    /// Create a timing that runs every minute
    pub fn every_minute() -> Self {
        Self {
            years: None,
            months: None,
            days: None,
            weekdays: None,
            hours: None,
            minutes: None,
        }
    }

    /// Create a timing that runs at specific minutes past the hour
    pub fn at_minutes(minutes: Vec<u32>) -> Self {
        Self {
            years: None,
            months: None,
            days: None,
            weekdays: None,
            hours: None,
            minutes: Some(minutes),
        }
    }

    /// Create a timing that runs at a specific hour and minute
    pub fn at_time(hour: u32, minute: u32) -> Self {
        Self {
            years: None,
            months: None,
            days: None,
            weekdays: None,
            hours: Some(vec![hour]),
            minutes: Some(vec![minute]),
        }
    }
}

/// Check if an event should run at the given datetime
pub fn check_timing<T: chrono::TimeZone>(timing: &Timing, dt: &DateTime<T>) -> bool {
    // If a field is None, it matches any value
    // If a field is Some, the current value must be in the list
    
    // Check year
    if let Some(years) = &timing.years {
        if !years.contains(&(dt.year() as u32)) {
            return false;
        }
    }
    
    // Check month (1-12)
    if let Some(months) = &timing.months {
        if !months.contains(&dt.month()) {
            return false;
        }
    }
    
    // Check day of month (1-31)
    if let Some(days) = &timing.days {
        if !days.contains(&dt.day()) {
            return false;
        }
    }
    
    // Check weekday (0=Sunday, 6=Saturday)
    if let Some(weekdays) = &timing.weekdays {
        let weekday = dt.weekday().num_days_from_sunday();
        if !weekdays.contains(&weekday) {
            return false;
        }
    }
    
    // Check hour (0-23)
    if let Some(hours) = &timing.hours {
        if !hours.contains(&dt.hour()) {
            return false;
        }
    }
    
    // Check minute (0-59)
    if let Some(minutes) = &timing.minutes {
        if !minutes.contains(&dt.minute()) {
            return false;
        }
    }
    
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, NaiveTime, Utc};

    #[test]
    fn test_every_minute() {
        let timing = Timing::every_minute();
        let dt = Utc::now();
        assert!(check_timing(&timing, &dt));
    }

    #[test]
    fn test_at_specific_minute() {
        let timing = Timing::at_minutes(vec![0, 15, 30, 45]);
        
        // Should match at minute 0
        let dt = NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_time(NaiveTime::from_hms_opt(12, 0, 0).unwrap())
            .and_utc();
        assert!(check_timing(&timing, &dt));
        
        // Should not match at minute 10
        let dt = NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_time(NaiveTime::from_hms_opt(12, 10, 0).unwrap())
            .and_utc();
        assert!(!check_timing(&timing, &dt));
        
        // Should match at minute 30
        let dt = NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_time(NaiveTime::from_hms_opt(12, 30, 0).unwrap())
            .and_utc();
        assert!(check_timing(&timing, &dt));
    }

    #[test]
    fn test_at_specific_time() {
        let timing = Timing::at_time(14, 30); // 2:30 PM
        
        // Should match
        let dt = NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_time(NaiveTime::from_hms_opt(14, 30, 0).unwrap())
            .and_utc();
        assert!(check_timing(&timing, &dt));
        
        // Should not match (wrong hour)
        let dt = NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_time(NaiveTime::from_hms_opt(15, 30, 0).unwrap())
            .and_utc();
        assert!(!check_timing(&timing, &dt));
        
        // Should not match (wrong minute)
        let dt = NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_time(NaiveTime::from_hms_opt(14, 45, 0).unwrap())
            .and_utc();
        assert!(!check_timing(&timing, &dt));
    }

    #[test]
    fn test_weekday_timing() {
        // Monday through Friday (1=Monday through 5=Friday)
        let timing = Timing {
            years: None,
            months: None,
            days: None,
            weekdays: Some(vec![1, 2, 3, 4, 5]), // Mon-Fri
            hours: None,
            minutes: None,
        };
        
        // Monday, 2024-01-01 is a Monday
        let dt = NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_time(NaiveTime::from_hms_opt(12, 0, 0).unwrap())
            .and_utc();
        assert!(check_timing(&timing, &dt));
        
        // Sunday, 2023-12-31 is a Sunday
        let dt = NaiveDate::from_ymd_opt(2023, 12, 31)
            .unwrap()
            .and_time(NaiveTime::from_hms_opt(12, 0, 0).unwrap())
            .and_utc();
        assert!(!check_timing(&timing, &dt));
    }
}
