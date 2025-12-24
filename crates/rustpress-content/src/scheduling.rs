//! # Content Scheduling with Timezone Support
//!
//! Advanced content scheduling with timezone awareness.
//!
//! Features:
//! - Timezone-aware scheduling
//! - Recurring publication schedules
//! - Publication queue management
//! - Schedule conflict detection

use chrono::{DateTime, Datelike, Duration, NaiveDateTime, NaiveTime, TimeZone, Utc, Weekday};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

/// Scheduled publication entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledPublication {
    /// Unique schedule ID
    pub id: Uuid,

    /// Post ID to publish
    pub post_id: i64,

    /// Scheduled datetime (UTC)
    pub scheduled_utc: DateTime<Utc>,

    /// Original timezone of scheduling
    pub timezone: String,

    /// Scheduled datetime in original timezone
    pub scheduled_local: String,

    /// Who scheduled this
    pub scheduled_by: i64,

    /// When this schedule was created
    pub created_at: DateTime<Utc>,

    /// Status after publication
    pub publish_status: String,

    /// Schedule type
    pub schedule_type: ScheduleType,

    /// Recurring schedule config (if applicable)
    pub recurring: Option<RecurringSchedule>,

    /// Whether schedule is active
    pub active: bool,

    /// Notification settings
    pub notifications: ScheduleNotifications,
}

impl ScheduledPublication {
    pub fn new(post_id: i64, scheduled: DateTime<Utc>, timezone: &str, scheduled_by: i64) -> Self {
        let tz: Tz = timezone.parse().unwrap_or(Tz::UTC);
        let local = scheduled.with_timezone(&tz);

        Self {
            id: Uuid::new_v4(),
            post_id,
            scheduled_utc: scheduled,
            timezone: timezone.to_string(),
            scheduled_local: local.format("%Y-%m-%d %H:%M:%S").to_string(),
            scheduled_by,
            created_at: Utc::now(),
            publish_status: "publish".to_string(),
            schedule_type: ScheduleType::OneTime,
            recurring: None,
            active: true,
            notifications: ScheduleNotifications::default(),
        }
    }

    /// Create from local datetime
    pub fn from_local(
        post_id: i64,
        local_datetime: &str,
        timezone: &str,
        scheduled_by: i64,
    ) -> Result<Self, SchedulingError> {
        let tz: Tz = timezone.parse()
            .map_err(|_| SchedulingError::InvalidTimezone(timezone.to_string()))?;

        let naive = NaiveDateTime::parse_from_str(local_datetime, "%Y-%m-%d %H:%M:%S")
            .map_err(|_| SchedulingError::InvalidDateTime(local_datetime.to_string()))?;

        let local = tz.from_local_datetime(&naive)
            .single()
            .ok_or_else(|| SchedulingError::AmbiguousTime(local_datetime.to_string()))?;

        let utc = local.with_timezone(&Utc);

        Ok(Self::new(post_id, utc, timezone, scheduled_by))
    }

    /// Check if schedule is due
    pub fn is_due(&self) -> bool {
        self.active && Utc::now() >= self.scheduled_utc
    }

    /// Time until scheduled publication
    pub fn time_until(&self) -> Option<Duration> {
        if self.is_due() {
            None
        } else {
            Some(self.scheduled_utc - Utc::now())
        }
    }

    /// Format time until in human readable form
    pub fn time_until_display(&self) -> String {
        match self.time_until() {
            None => "Due now".to_string(),
            Some(duration) => {
                let days = duration.num_days();
                let hours = duration.num_hours() % 24;
                let minutes = duration.num_minutes() % 60;

                if days > 0 {
                    format!("{} days, {} hours", days, hours)
                } else if hours > 0 {
                    format!("{} hours, {} minutes", hours, minutes)
                } else {
                    format!("{} minutes", minutes)
                }
            }
        }
    }

    /// Get display time in specified timezone
    pub fn display_time(&self, display_tz: &str) -> String {
        let tz: Tz = display_tz.parse().unwrap_or(Tz::UTC);
        let local = self.scheduled_utc.with_timezone(&tz);
        local.format("%Y-%m-%d %H:%M %Z").to_string()
    }
}

/// Schedule type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScheduleType {
    /// One-time publication
    OneTime,

    /// Recurring schedule
    Recurring,

    /// Series schedule (part of a sequence)
    Series,
}

/// Recurring schedule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurringSchedule {
    /// Recurrence pattern
    pub pattern: RecurrencePattern,

    /// Interval (e.g., every 2 weeks)
    pub interval: u32,

    /// Specific days (for weekly)
    pub days_of_week: Vec<Weekday>,

    /// Specific day of month (for monthly)
    pub day_of_month: Option<u8>,

    /// Time of day
    pub time: NaiveTime,

    /// Start date
    pub start_date: DateTime<Utc>,

    /// End date (None = indefinite)
    pub end_date: Option<DateTime<Utc>>,

    /// Maximum occurrences (None = unlimited)
    pub max_occurrences: Option<u32>,

    /// Current occurrence count
    pub occurrence_count: u32,
}

/// Recurrence pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecurrencePattern {
    Daily,
    Weekly,
    BiWeekly,
    Monthly,
    Yearly,
    Custom,
}

impl RecurringSchedule {
    pub fn daily(time: NaiveTime) -> Self {
        Self {
            pattern: RecurrencePattern::Daily,
            interval: 1,
            days_of_week: Vec::new(),
            day_of_month: None,
            time,
            start_date: Utc::now(),
            end_date: None,
            max_occurrences: None,
            occurrence_count: 0,
        }
    }

    pub fn weekly(time: NaiveTime, days: Vec<Weekday>) -> Self {
        Self {
            pattern: RecurrencePattern::Weekly,
            interval: 1,
            days_of_week: days,
            day_of_month: None,
            time,
            start_date: Utc::now(),
            end_date: None,
            max_occurrences: None,
            occurrence_count: 0,
        }
    }

    pub fn monthly(time: NaiveTime, day: u8) -> Self {
        Self {
            pattern: RecurrencePattern::Monthly,
            interval: 1,
            days_of_week: Vec::new(),
            day_of_month: Some(day.min(31)),
            time,
            start_date: Utc::now(),
            end_date: None,
            max_occurrences: None,
            occurrence_count: 0,
        }
    }

    /// Check if schedule has more occurrences
    pub fn has_more(&self) -> bool {
        if let Some(max) = self.max_occurrences {
            if self.occurrence_count >= max {
                return false;
            }
        }

        if let Some(end) = self.end_date {
            if Utc::now() > end {
                return false;
            }
        }

        true
    }

    /// Calculate next occurrence
    pub fn next_occurrence(&self, after: DateTime<Utc>, timezone: &str) -> Option<DateTime<Utc>> {
        if !self.has_more() {
            return None;
        }

        let tz: Tz = timezone.parse().unwrap_or(Tz::UTC);
        let local = after.with_timezone(&tz);

        let next = match self.pattern {
            RecurrencePattern::Daily => {
                let next_date = local.date_naive() + Duration::days(self.interval as i64);
                tz.from_local_datetime(&next_date.and_time(self.time))
                    .single()
                    .map(|dt| dt.with_timezone(&Utc))
            }
            RecurrencePattern::Weekly => {
                // Find next matching day
                let mut next_date = local.date_naive() + Duration::days(1);
                for _ in 0..8 {
                    if self.days_of_week.contains(&next_date.weekday()) {
                        return tz.from_local_datetime(&next_date.and_time(self.time))
                            .single()
                            .map(|dt| dt.with_timezone(&Utc));
                    }
                    next_date += Duration::days(1);
                }
                None
            }
            RecurrencePattern::BiWeekly => {
                let next_date = local.date_naive() + Duration::weeks(2);
                tz.from_local_datetime(&next_date.and_time(self.time))
                    .single()
                    .map(|dt| dt.with_timezone(&Utc))
            }
            RecurrencePattern::Monthly => {
                let day = self.day_of_month.unwrap_or(1);
                let mut next_month = local.month() + 1;
                let mut next_year = local.year();

                if next_month > 12 {
                    next_month = 1;
                    next_year += 1;
                }

                // Handle day overflow for short months
                let days_in_month = days_in_month(next_year, next_month);
                let actual_day = day.min(days_in_month);

                chrono::NaiveDate::from_ymd_opt(next_year, next_month, actual_day as u32)
                    .and_then(|date| {
                        tz.from_local_datetime(&date.and_time(self.time))
                            .single()
                            .map(|dt| dt.with_timezone(&Utc))
                    })
            }
            RecurrencePattern::Yearly => {
                let next_date = local.date_naive() + Duration::days(365);
                tz.from_local_datetime(&next_date.and_time(self.time))
                    .single()
                    .map(|dt| dt.with_timezone(&Utc))
            }
            RecurrencePattern::Custom => None,
        };

        // Check against end date
        next.filter(|dt| self.end_date.map(|end| *dt <= end).unwrap_or(true))
    }
}

/// Get days in a month
fn days_in_month(year: i32, month: u32) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

/// Schedule notification settings
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScheduleNotifications {
    /// Notify author before publication
    pub notify_author: bool,

    /// Minutes before to notify
    pub notify_minutes_before: u32,

    /// Notify on publication
    pub notify_on_publish: bool,

    /// Email addresses to notify
    pub notify_emails: Vec<String>,
}

/// Scheduling error
#[derive(Debug, Clone)]
pub enum SchedulingError {
    InvalidTimezone(String),
    InvalidDateTime(String),
    AmbiguousTime(String),
    PastDateTime,
    ConflictDetected(String),
}

impl std::fmt::Display for SchedulingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidTimezone(tz) => write!(f, "Invalid timezone: {}", tz),
            Self::InvalidDateTime(dt) => write!(f, "Invalid datetime: {}", dt),
            Self::AmbiguousTime(dt) => write!(f, "Ambiguous time (DST transition): {}", dt),
            Self::PastDateTime => write!(f, "Cannot schedule in the past"),
            Self::ConflictDetected(msg) => write!(f, "Schedule conflict: {}", msg),
        }
    }
}

impl std::error::Error for SchedulingError {}

/// Publication scheduler
pub struct PublicationScheduler {
    /// Scheduled publications
    schedules: HashMap<Uuid, ScheduledPublication>,

    /// Post ID to schedule ID mapping
    post_schedules: HashMap<i64, Vec<Uuid>>,

    /// Default timezone
    default_timezone: String,

    /// Minimum schedule lead time (minutes)
    min_lead_time: u32,

    /// Allow scheduling in the past
    allow_past: bool,
}

impl Default for PublicationScheduler {
    fn default() -> Self {
        Self {
            schedules: HashMap::new(),
            post_schedules: HashMap::new(),
            default_timezone: "UTC".to_string(),
            min_lead_time: 5,
            allow_past: false,
        }
    }
}

impl PublicationScheduler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_timezone(mut self, tz: &str) -> Self {
        self.default_timezone = tz.to_string();
        self
    }

    /// Schedule a publication
    pub fn schedule(&mut self, schedule: ScheduledPublication) -> Result<Uuid, SchedulingError> {
        // Validate not in past
        if !self.allow_past && schedule.scheduled_utc < Utc::now() {
            return Err(SchedulingError::PastDateTime);
        }

        // Check minimum lead time
        let min_time = Utc::now() + Duration::minutes(self.min_lead_time as i64);
        if !self.allow_past && schedule.scheduled_utc < min_time {
            return Err(SchedulingError::PastDateTime);
        }

        let id = schedule.id;
        let post_id = schedule.post_id;

        self.schedules.insert(id, schedule);
        self.post_schedules
            .entry(post_id)
            .or_insert_with(Vec::new)
            .push(id);

        Ok(id)
    }

    /// Cancel a schedule
    pub fn cancel(&mut self, schedule_id: &Uuid) -> Option<ScheduledPublication> {
        if let Some(schedule) = self.schedules.remove(schedule_id) {
            if let Some(post_schedules) = self.post_schedules.get_mut(&schedule.post_id) {
                post_schedules.retain(|id| id != schedule_id);
            }
            Some(schedule)
        } else {
            None
        }
    }

    /// Get schedules for a post
    pub fn get_post_schedules(&self, post_id: i64) -> Vec<&ScheduledPublication> {
        self.post_schedules
            .get(&post_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.schedules.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get due schedules
    pub fn get_due(&self) -> Vec<&ScheduledPublication> {
        self.schedules.values()
            .filter(|s| s.is_due())
            .collect()
    }

    /// Process due schedules (returns post IDs ready for publication)
    pub fn process_due(&mut self) -> Vec<i64> {
        let due: Vec<Uuid> = self.schedules.values()
            .filter(|s| s.is_due())
            .map(|s| s.id)
            .collect();

        let mut post_ids = Vec::new();

        for id in due {
            if let Some(mut schedule) = self.schedules.remove(&id) {
                post_ids.push(schedule.post_id);

                // Handle recurring
                if let Some(ref mut recurring) = schedule.recurring {
                    recurring.occurrence_count += 1;

                    if let Some(next) = recurring.next_occurrence(schedule.scheduled_utc, &schedule.timezone) {
                        let mut next_schedule = schedule.clone();
                        next_schedule.id = Uuid::new_v4();
                        next_schedule.scheduled_utc = next;
                        next_schedule.created_at = Utc::now();

                        let _ = self.schedule(next_schedule);
                    }
                }

                // Cleanup post_schedules mapping
                if let Some(post_schedules) = self.post_schedules.get_mut(&schedule.post_id) {
                    post_schedules.retain(|sid| *sid != id);
                }
            }
        }

        post_ids
    }

    /// Get upcoming schedules
    pub fn get_upcoming(&self, limit: usize) -> Vec<&ScheduledPublication> {
        let mut schedules: Vec<&ScheduledPublication> = self.schedules.values()
            .filter(|s| s.active && !s.is_due())
            .collect();

        schedules.sort_by_key(|s| s.scheduled_utc);
        schedules.truncate(limit);
        schedules
    }

    /// Check for conflicts (same post scheduled at similar time)
    pub fn check_conflicts(&self, post_id: i64, scheduled: DateTime<Utc>, window_minutes: i64) -> Vec<&ScheduledPublication> {
        let window_start = scheduled - Duration::minutes(window_minutes);
        let window_end = scheduled + Duration::minutes(window_minutes);

        self.get_post_schedules(post_id)
            .into_iter()
            .filter(|s| s.scheduled_utc >= window_start && s.scheduled_utc <= window_end)
            .collect()
    }
}

/// Get list of common timezones
pub fn get_common_timezones() -> Vec<(&'static str, &'static str)> {
    vec![
        ("UTC", "UTC (Coordinated Universal Time)"),
        ("America/New_York", "Eastern Time (US & Canada)"),
        ("America/Chicago", "Central Time (US & Canada)"),
        ("America/Denver", "Mountain Time (US & Canada)"),
        ("America/Los_Angeles", "Pacific Time (US & Canada)"),
        ("America/Sao_Paulo", "Brasilia Time"),
        ("Europe/London", "London (GMT/BST)"),
        ("Europe/Paris", "Paris (CET/CEST)"),
        ("Europe/Berlin", "Berlin (CET/CEST)"),
        ("Europe/Moscow", "Moscow Time"),
        ("Asia/Dubai", "Dubai (GST)"),
        ("Asia/Kolkata", "India Standard Time"),
        ("Asia/Shanghai", "China Standard Time"),
        ("Asia/Tokyo", "Japan Standard Time"),
        ("Australia/Sydney", "Sydney (AEST/AEDT)"),
    ]
}

/// Convert datetime between timezones
pub fn convert_timezone(datetime: DateTime<Utc>, to_tz: &str) -> Option<String> {
    let tz: Tz = to_tz.parse().ok()?;
    let local = datetime.with_timezone(&tz);
    Some(local.format("%Y-%m-%d %H:%M:%S %Z").to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schedule_creation() {
        let schedule = ScheduledPublication::new(1, Utc::now() + Duration::hours(1), "America/New_York", 1);
        assert!(!schedule.is_due());
        assert!(schedule.active);
    }

    #[test]
    fn test_from_local() {
        let result = ScheduledPublication::from_local(
            1,
            "2025-06-15 10:00:00",
            "America/New_York",
            1,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_timezone() {
        let result = ScheduledPublication::from_local(
            1,
            "2025-06-15 10:00:00",
            "Invalid/Timezone",
            1,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_recurring_schedule() {
        let time = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
        let recurring = RecurringSchedule::daily(time);

        assert!(recurring.has_more());
        let next = recurring.next_occurrence(Utc::now(), "UTC");
        assert!(next.is_some());
    }

    #[test]
    fn test_scheduler() {
        let mut scheduler = PublicationScheduler::new();

        let schedule = ScheduledPublication::new(
            1,
            Utc::now() + Duration::hours(1),
            "UTC",
            1,
        );

        let result = scheduler.schedule(schedule);
        assert!(result.is_ok());

        let upcoming = scheduler.get_upcoming(10);
        assert_eq!(upcoming.len(), 1);
    }

    #[test]
    fn test_timezone_conversion() {
        let utc = Utc::now();
        let ny = convert_timezone(utc, "America/New_York");
        assert!(ny.is_some());
        assert!(ny.unwrap().contains("EDT") || ny.unwrap().contains("EST"));
    }
}
