use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
};

pub fn calculate_layout(area: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(area);

    (chunks[0], chunks[1])
}

pub fn calculate_workflow_layout(area: Rect, repo_count: usize) -> Vec<Rect> {
    let constraints = vec![Constraint::Length(6); repo_count];
    Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area)
        .to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_layout_normal_size() {
        let area = Rect::new(0, 0, 80, 24);
        let (main, status) = calculate_layout(area);
        
        assert_eq!(main.x, 0);
        assert_eq!(main.y, 0);
        assert_eq!(main.width, 80);
        assert_eq!(main.height, 21); // 24 - 3 for status
        
        assert_eq!(status.x, 0);
        assert_eq!(status.y, 21);
        assert_eq!(status.width, 80);
        assert_eq!(status.height, 3);
    }

    #[test]
    fn test_calculate_layout_minimum_size() {
        let area = Rect::new(0, 0, 10, 13);
        let (main, status) = calculate_layout(area);
        
        assert_eq!(main.height, 10); // Min(10) constraint
        assert_eq!(status.height, 3);
        assert_eq!(main.height + status.height, area.height);
    }

    #[test]
    fn test_calculate_layout_small_terminal() {
        let area = Rect::new(0, 0, 5, 8);
        let (main, status) = calculate_layout(area);
        
        // With very small terminal, ratatui prioritizes the Min constraint
        // but may not be able to satisfy both constraints fully
        let total_height = main.height + status.height;
        assert_eq!(total_height, area.height);
        
        // Status should still get some space, but maybe not full 3 lines
        assert!(status.height <= 3);
    }

    #[test]
    fn test_calculate_layout_large_terminal() {
        let area = Rect::new(0, 0, 200, 100);
        let (main, status) = calculate_layout(area);
        
        assert_eq!(status.height, 3);
        assert_eq!(main.height, 97); // 100 - 3
        assert_eq!(main.width, 200);
        assert_eq!(status.width, 200);
    }

    #[test]
    fn test_calculate_layout_position_preservation() {
        let area = Rect::new(10, 5, 80, 24);
        let (main, status) = calculate_layout(area);
        
        assert_eq!(main.x, 10);
        assert_eq!(main.y, 5);
        assert_eq!(status.x, 10);
        assert_eq!(status.y, 26); // 5 + 21 (main height)
    }

    #[test]
    fn test_calculate_workflow_layout_single_repo() {
        let area = Rect::new(0, 0, 80, 24);
        let layouts = calculate_workflow_layout(area, 1);
        
        assert_eq!(layouts.len(), 1);
        assert_eq!(layouts[0].height, 6);
        assert_eq!(layouts[0].width, 80);
        assert_eq!(layouts[0].x, 0);
        assert_eq!(layouts[0].y, 0);
    }

    #[test]
    fn test_calculate_workflow_layout_multiple_repos() {
        let area = Rect::new(0, 0, 80, 24);
        let layouts = calculate_workflow_layout(area, 3);
        
        assert_eq!(layouts.len(), 3);
        
        for (i, layout) in layouts.iter().enumerate() {
            assert_eq!(layout.height, 6);
            assert_eq!(layout.width, 80);
            assert_eq!(layout.x, 0);
            assert_eq!(layout.y, i as u16 * 6);
        }
    }

    #[test]
    fn test_calculate_workflow_layout_zero_repos() {
        let area = Rect::new(0, 0, 80, 24);
        let layouts = calculate_workflow_layout(area, 0);
        
        assert_eq!(layouts.len(), 0);
    }

    #[test]
    fn test_calculate_workflow_layout_many_repos() {
        let area = Rect::new(0, 0, 80, 24);
        let layouts = calculate_workflow_layout(area, 10);
        
        assert_eq!(layouts.len(), 10);
        
        // With 24 height and 10 constraints of 6 each (total 60), 
        // ratatui will distribute proportionally
        let total_height: u16 = layouts.iter().map(|l| l.height).sum();
        assert_eq!(total_height, area.height);
        
        // Check that layouts are positioned correctly
        for (i, layout) in layouts.iter().enumerate() {
            if i == 0 {
                assert_eq!(layout.y, 0);
            } else {
                assert!(layout.y > layouts[i-1].y);
            }
        }
    }

    #[test]
    fn test_calculate_workflow_layout_insufficient_height() {
        let area = Rect::new(0, 0, 80, 10);
        let layouts = calculate_workflow_layout(area, 3);
        
        // Should still create 3 layouts, but ratatui will adjust heights to fit
        assert_eq!(layouts.len(), 3);
        // With 10 height and 3 constraints of 6 each, ratatui distributes proportionally
        // The actual heights will be adjusted to fit within the area
        let total_height: u16 = layouts.iter().map(|l| l.height).sum();
        assert_eq!(total_height, area.height);
    }

    #[test]
    fn test_calculate_workflow_layout_position_preservation() {
        let area = Rect::new(5, 10, 80, 24);
        let layouts = calculate_workflow_layout(area, 2);
        
        assert_eq!(layouts.len(), 2);
        assert_eq!(layouts[0].x, 5);
        assert_eq!(layouts[0].y, 10);
        assert_eq!(layouts[1].x, 5);
        assert_eq!(layouts[1].y, 16); // 10 + 6
    }

    #[test]
    fn test_calculate_workflow_layout_narrow_width() {
        let area = Rect::new(0, 0, 20, 24);
        let layouts = calculate_workflow_layout(area, 2);
        
        assert_eq!(layouts.len(), 2);
        for layout in &layouts {
            assert_eq!(layout.width, 20);
        }
    }

    #[test]
    fn test_layout_constraints_boundary() {
        // Test with exactly minimum height
        let area = Rect::new(0, 0, 80, 13);
        let (main, status) = calculate_layout(area);
        
        assert_eq!(main.height, 10); // Exactly the minimum
        assert_eq!(status.height, 3);
        
        // Test one less than minimum - ratatui will adjust proportionally
        let area = Rect::new(0, 0, 80, 12);
        let (main, status) = calculate_layout(area);
        
        // Total should equal area height, but distribution may vary
        let total_height = main.height + status.height;
        assert_eq!(total_height, area.height);
        assert!(status.height <= 3);
    }
}