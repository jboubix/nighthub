use chrono::{DateTime, Utc};
use std::time::Duration;

pub fn format_relative_time(time: DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(time);

    if duration.num_days() > 0 {
        format!("{}d ago", duration.num_days())
    } else if duration.num_hours() > 0 {
        format!("{}h ago", duration.num_hours())
    } else if duration.num_minutes() > 0 {
        format!("{}m ago", duration.num_minutes())
    } else {
        "Just now".to_string()
    }
}

pub fn duration_to_human_readable(duration: Duration) -> String {
    let seconds = duration.as_secs();
    let minutes = seconds / 60;
    let hours = minutes / 60;
    let days = hours / 24;

    if days > 0 {
        format!("{}d {}h", days, hours % 24)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes % 60)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds % 60)
    } else {
        format!("{}s", seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration as ChronoDuration;

    #[test]
    fn test_format_relative_time_just_now() {
        let now = Utc::now();
        let result = format_relative_time(now);
        assert_eq!(result, "Just now");
    }

    #[test]
    fn test_format_relative_time_seconds_ago() {
        let now = Utc::now();
        let time_30_seconds_ago = now - ChronoDuration::seconds(30);
        let result = format_relative_time(time_30_seconds_ago);
        assert_eq!(result, "Just now");
    }

    #[test]
    fn test_format_relative_time_one_minute_ago() {
        let now = Utc::now();
        let time_1_minute_ago = now - ChronoDuration::minutes(1);
        let result = format_relative_time(time_1_minute_ago);
        assert_eq!(result, "1m ago");
    }

    #[test]
    fn test_format_relative_time_minutes_ago() {
        let now = Utc::now();
        let time_30_minutes_ago = now - ChronoDuration::minutes(30);
        let result = format_relative_time(time_30_minutes_ago);
        assert_eq!(result, "30m ago");
    }

    #[test]
    fn test_format_relative_time_one_hour_ago() {
        let now = Utc::now();
        let time_1_hour_ago = now - ChronoDuration::hours(1);
        let result = format_relative_time(time_1_hour_ago);
        assert_eq!(result, "1h ago");
    }

    #[test]
    fn test_format_relative_time_hours_ago() {
        let now = Utc::now();
        let time_5_hours_ago = now - ChronoDuration::hours(5);
        let result = format_relative_time(time_5_hours_ago);
        assert_eq!(result, "5h ago");
    }

    #[test]
    fn test_format_relative_time_one_day_ago() {
        let now = Utc::now();
        let time_1_day_ago = now - ChronoDuration::days(1);
        let result = format_relative_time(time_1_day_ago);
        assert_eq!(result, "1d ago");
    }

    #[test]
    fn test_format_relative_time_days_ago() {
        let now = Utc::now();
        let time_7_days_ago = now - ChronoDuration::days(7);
        let result = format_relative_time(time_7_days_ago);
        assert_eq!(result, "7d ago");
    }

    #[test]
    fn test_format_relative_time_boundary_conditions() {
        let now = Utc::now();
        
        // Test 59 seconds - should be "Just now"
        let time_59_seconds_ago = now - ChronoDuration::seconds(59);
        let result = format_relative_time(time_59_seconds_ago);
        assert_eq!(result, "Just now");
        
        // Test 60 seconds - should be "1m ago"
        let time_60_seconds_ago = now - ChronoDuration::seconds(60);
        let result = format_relative_time(time_60_seconds_ago);
        assert_eq!(result, "1m ago");
        
        // Test 59 minutes - should be "59m ago"
        let time_59_minutes_ago = now - ChronoDuration::minutes(59);
        let result = format_relative_time(time_59_minutes_ago);
        assert_eq!(result, "59m ago");
        
        // Test 60 minutes - should be "1h ago"
        let time_60_minutes_ago = now - ChronoDuration::minutes(60);
        let result = format_relative_time(time_60_minutes_ago);
        assert_eq!(result, "1h ago");
        
        // Test 23 hours - should be "23h ago"
        let time_23_hours_ago = now - ChronoDuration::hours(23);
        let result = format_relative_time(time_23_hours_ago);
        assert_eq!(result, "23h ago");
        
        // Test 24 hours - should be "1d ago"
        let time_24_hours_ago = now - ChronoDuration::hours(24);
        let result = format_relative_time(time_24_hours_ago);
        assert_eq!(result, "1d ago");
    }

    #[test]
    fn test_format_relative_time_future_time() {
        let now = Utc::now();
        let time_future = now + ChronoDuration::minutes(5);
        let result = format_relative_time(time_future);
        
        // Future times should result in "Just now" since duration will be negative
        assert_eq!(result, "Just now");
    }

    #[test]
    fn test_duration_to_human_readable_zero_seconds() {
        let duration = Duration::from_secs(0);
        let result = duration_to_human_readable(duration);
        assert_eq!(result, "0s");
    }

    #[test]
    fn test_duration_to_human_readable_seconds_only() {
        let duration = Duration::from_secs(30);
        let result = duration_to_human_readable(duration);
        assert_eq!(result, "30s");
    }

    #[test]
    fn test_duration_to_human_readable_one_minute() {
        let duration = Duration::from_secs(60);
        let result = duration_to_human_readable(duration);
        assert_eq!(result, "1m 0s");
    }

    #[test]
    fn test_duration_to_human_readable_minutes_and_seconds() {
        let duration = Duration::from_secs(125); // 2 minutes 5 seconds
        let result = duration_to_human_readable(duration);
        assert_eq!(result, "2m 5s");
    }

    #[test]
    fn test_duration_to_human_readable_one_hour() {
        let duration = Duration::from_secs(3600);
        let result = duration_to_human_readable(duration);
        assert_eq!(result, "1h 0m");
    }

    #[test]
    fn test_duration_to_human_readable_hours_and_minutes() {
        let duration = Duration::from_secs(3665); // 1 hour 1 minute 5 seconds
        let result = duration_to_human_readable(duration);
        assert_eq!(result, "1h 1m");
    }

    #[test]
    fn test_duration_to_human_readable_one_day() {
        let duration = Duration::from_secs(86400);
        let result = duration_to_human_readable(duration);
        assert_eq!(result, "1d 0h");
    }

    #[test]
    fn test_duration_to_human_readable_days_and_hours() {
        let duration = Duration::from_secs(90000); // 1 day 1 hour
        let result = duration_to_human_readable(duration);
        assert_eq!(result, "1d 1h");
    }

    #[test]
    fn test_duration_to_human_readable_complex_duration() {
        let duration = Duration::from_secs(93784); // 1 day 2 hours 3 minutes 4 seconds
        let result = duration_to_human_readable(duration);
        assert_eq!(result, "1d 2h");
    }

    #[test]
    fn test_duration_to_human_readable_large_duration() {
        let duration = Duration::from_secs(259200); // 3 days
        let result = duration_to_human_readable(duration);
        assert_eq!(result, "3d 0h");
    }

    #[test]
    fn test_duration_to_human_readable_boundary_conditions() {
        // Test 59 seconds
        let duration = Duration::from_secs(59);
        let result = duration_to_human_readable(duration);
        assert_eq!(result, "59s");
        
        // Test 60 seconds (1 minute)
        let duration = Duration::from_secs(60);
        let result = duration_to_human_readable(duration);
        assert_eq!(result, "1m 0s");
        
        // Test 59 minutes 59 seconds
        let duration = Duration::from_secs(3599);
        let result = duration_to_human_readable(duration);
        assert_eq!(result, "59m 59s");
        
        // Test 60 minutes (1 hour)
        let duration = Duration::from_secs(3600);
        let result = duration_to_human_readable(duration);
        assert_eq!(result, "1h 0m");
        
        // Test 23 hours 59 minutes
        let duration = Duration::from_secs(86399);
        let result = duration_to_human_readable(duration);
        assert_eq!(result, "23h 59m");
        
        // Test 24 hours (1 day)
        let duration = Duration::from_secs(86400);
        let result = duration_to_human_readable(duration);
        assert_eq!(result, "1d 0h");
    }

    #[test]
    fn test_duration_to_human_readable_edge_cases() {
        // Test exactly 1 day 12 hours
        let duration = Duration::from_secs(129600); // 86400 + 43200
        let result = duration_to_human_readable(duration);
        assert_eq!(result, "1d 12h");
        
        // Test 2 days 23 hours
        let duration = Duration::from_secs(216000); // 2*86400 + 23*3600
        let result = duration_to_human_readable(duration);
        assert_eq!(result, "2d 12h"); // 216000 / 3600 = 60 hours, 60 / 24 = 2 days, 60 % 24 = 12 hours
        
        // Test large number of seconds
        let duration = Duration::from_secs(1000000);
        let result = duration_to_human_readable(duration);
        // Should show days and remaining hours
        assert!(result.contains("d"));
        assert!(result.contains("h"));
    }
}