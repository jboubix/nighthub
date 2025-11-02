use crate::github::models::{WorkflowStatus, WorkflowConclusion};

pub fn get_status_icon(status: &WorkflowStatus) -> &'static str {
    match status {
        WorkflowStatus::Queued => "⏳",
        WorkflowStatus::InProgress => "🔄",
        WorkflowStatus::Completed => "✅",  // Default, conclusion handled separately
    }
}

pub fn get_conclusion_icon(conclusion: &Option<WorkflowConclusion>) -> &'static str {
    match conclusion {
        Some(WorkflowConclusion::Success) => "✅",
        Some(WorkflowConclusion::Failure) => "❌",
        Some(WorkflowConclusion::Cancelled) => "⏹️",
        Some(WorkflowConclusion::Skipped) => "⏭️",
        Some(WorkflowConclusion::TimedOut) => "⏰",
        None => "?",
    }
}

pub fn get_status_text(status: &WorkflowStatus, conclusion: &Option<WorkflowConclusion>) -> &'static str {
    match (status, conclusion) {
        (WorkflowStatus::Queued, _) => "Queued",
        (WorkflowStatus::InProgress, _) => "In Progress",
        (WorkflowStatus::Completed, Some(con)) => match con {
            WorkflowConclusion::Success => "Success",
            WorkflowConclusion::Failure => "Failure",
            WorkflowConclusion::Cancelled => "Cancelled",
            WorkflowConclusion::Skipped => "Skipped",
            WorkflowConclusion::TimedOut => "Timed Out",
        },
        (WorkflowStatus::Completed, None) => "Completed",

    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::github::models::{WorkflowStatus, WorkflowConclusion};

    #[test]
    fn test_get_status_icon_queued() {
        let icon = get_status_icon(&WorkflowStatus::Queued);
        assert_eq!(icon, "⏳");
    }

    #[test]
    fn test_get_status_icon_in_progress() {
        let icon = get_status_icon(&WorkflowStatus::InProgress);
        assert_eq!(icon, "🔄");
    }

    #[test]
    fn test_get_status_icon_completed() {
        let icon = get_status_icon(&WorkflowStatus::Completed);
        assert_eq!(icon, "✅");
    }

    #[test]
    fn test_get_conclusion_icon_success() {
        let icon = get_conclusion_icon(&Some(WorkflowConclusion::Success));
        assert_eq!(icon, "✅");
    }

    #[test]
    fn test_get_conclusion_icon_failure() {
        let icon = get_conclusion_icon(&Some(WorkflowConclusion::Failure));
        assert_eq!(icon, "❌");
    }

    #[test]
    fn test_get_conclusion_icon_cancelled() {
        let icon = get_conclusion_icon(&Some(WorkflowConclusion::Cancelled));
        assert_eq!(icon, "⏹️");
    }

    #[test]
    fn test_get_conclusion_icon_skipped() {
        let icon = get_conclusion_icon(&Some(WorkflowConclusion::Skipped));
        assert_eq!(icon, "⏭️");
    }

    #[test]
    fn test_get_conclusion_icon_timed_out() {
        let icon = get_conclusion_icon(&Some(WorkflowConclusion::TimedOut));
        assert_eq!(icon, "⏰");
    }

    #[test]
    fn test_get_conclusion_icon_none() {
        let icon = get_conclusion_icon(&None);
        assert_eq!(icon, "?");
    }

    #[test]
    fn test_get_status_text_queued() {
        let text = get_status_text(&WorkflowStatus::Queued, &None);
        assert_eq!(text, "Queued");
    }

    #[test]
    fn test_get_status_text_in_progress() {
        let text = get_status_text(&WorkflowStatus::InProgress, &None);
        assert_eq!(text, "In Progress");
    }

    #[test]
    fn test_get_status_text_completed_success() {
        let text = get_status_text(&WorkflowStatus::Completed, &Some(WorkflowConclusion::Success));
        assert_eq!(text, "Success");
    }

    #[test]
    fn test_get_status_text_completed_failure() {
        let text = get_status_text(&WorkflowStatus::Completed, &Some(WorkflowConclusion::Failure));
        assert_eq!(text, "Failure");
    }

    #[test]
    fn test_get_status_text_completed_none() {
        let text = get_status_text(&WorkflowStatus::Completed, &None);
        assert_eq!(text, "Completed");
    }

    #[test]
    fn test_all_status_icons_unique() {
        let queued_icon = get_status_icon(&WorkflowStatus::Queued);
        let in_progress_icon = get_status_icon(&WorkflowStatus::InProgress);
        let completed_icon = get_status_icon(&WorkflowStatus::Completed);
        
        assert_ne!(queued_icon, in_progress_icon);
        assert_ne!(in_progress_icon, completed_icon);
        assert_ne!(queued_icon, completed_icon);
    }

    #[test]
    fn test_all_conclusion_icons_unique() {
        let success_icon = get_conclusion_icon(&Some(WorkflowConclusion::Success));
        let failure_icon = get_conclusion_icon(&Some(WorkflowConclusion::Failure));
        let cancelled_icon = get_conclusion_icon(&Some(WorkflowConclusion::Cancelled));
        let skipped_icon = get_conclusion_icon(&Some(WorkflowConclusion::Skipped));
        let timed_out_icon = get_conclusion_icon(&Some(WorkflowConclusion::TimedOut));
        let none_icon = get_conclusion_icon(&None);
        
        // Ensure all icons are different
        let icons = vec![success_icon, failure_icon, cancelled_icon, skipped_icon, timed_out_icon, none_icon];
        for (i, icon1) in icons.iter().enumerate() {
            for (j, icon2) in icons.iter().enumerate() {
                if i != j {
                    assert_ne!(icon1, icon2, "Icons at positions {} and {} should be different", i, j);
                }
            }
        }
    }

    #[test]
    fn test_status_icons_non_empty() {
        let queued_icon = get_status_icon(&WorkflowStatus::Queued);
        let in_progress_icon = get_status_icon(&WorkflowStatus::InProgress);
        let completed_icon = get_status_icon(&WorkflowStatus::Completed);
        
        assert!(!queued_icon.is_empty());
        assert!(!in_progress_icon.is_empty());
        assert!(!completed_icon.is_empty());
    }

    #[test]
    fn test_conclusion_icons_non_empty() {
        let success_icon = get_conclusion_icon(&Some(WorkflowConclusion::Success));
        let failure_icon = get_conclusion_icon(&Some(WorkflowConclusion::Failure));
        let cancelled_icon = get_conclusion_icon(&Some(WorkflowConclusion::Cancelled));
        let skipped_icon = get_conclusion_icon(&Some(WorkflowConclusion::Skipped));
        let timed_out_icon = get_conclusion_icon(&Some(WorkflowConclusion::TimedOut));
        let none_icon = get_conclusion_icon(&None);
        
        assert!(!success_icon.is_empty());
        assert!(!failure_icon.is_empty());
        assert!(!cancelled_icon.is_empty());
        assert!(!skipped_icon.is_empty());
        assert!(!timed_out_icon.is_empty());
        assert!(!none_icon.is_empty());
    }

    #[test]
    fn test_status_text_all_combinations() {
        // Test all status/conclusion combinations
        let test_cases = vec![
            (WorkflowStatus::Queued, None, "Queued"),
            (WorkflowStatus::Queued, Some(WorkflowConclusion::Success), "Queued"),
            (WorkflowStatus::InProgress, None, "In Progress"),
            (WorkflowStatus::InProgress, Some(WorkflowConclusion::Failure), "In Progress"),
            (WorkflowStatus::Completed, None, "Completed"),
            (WorkflowStatus::Completed, Some(WorkflowConclusion::Success), "Success"),
            (WorkflowStatus::Completed, Some(WorkflowConclusion::Failure), "Failure"),
            (WorkflowStatus::Completed, Some(WorkflowConclusion::Cancelled), "Cancelled"),
            (WorkflowStatus::Completed, Some(WorkflowConclusion::Skipped), "Skipped"),
            (WorkflowStatus::Completed, Some(WorkflowConclusion::TimedOut), "Timed Out"),
        ];
        
        for (status, conclusion, expected_text) in test_cases {
            let text = get_status_text(&status, &conclusion);
            assert_eq!(text, expected_text, "Failed for status: {:?}, conclusion: {:?}", status, conclusion);
        }
    }
}
