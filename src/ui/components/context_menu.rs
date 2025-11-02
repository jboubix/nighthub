use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub struct ContextMenuComponent {
    pub selected_index: usize,
    pub items: Vec<String>,
}

impl ContextMenuComponent {
    pub fn new() -> Self {
        ContextMenuComponent {
            selected_index: 0,
            items: vec![
                "View Logs".to_string(),
                "Open in Browser".to_string(),
                "Close Menu".to_string(),
            ],
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let line = Span::styled(
                    item.clone(),
                    if index == self.selected_index {
                        Style::default().fg(Color::Black).bg(Color::White)
                    } else {
                        Style::default().fg(Color::White)
                    },
                );
                ListItem::new(vec![line.into()])
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Actions"))
            .highlight_style(Style::default().bg(Color::DarkGray));

        frame.render_widget(list, area);
    }

    pub fn next(&mut self) {
        self.selected_index = (self.selected_index + 1) % self.items.len();
    }

    pub fn previous(&mut self) {
        let len = self.items.len();
        self.selected_index = (self.selected_index + len - 1) % len;
    }

    pub fn get_selected_action(&self) -> &str {
        &self.items[self.selected_index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_menu_new() {
        let menu = ContextMenuComponent::new();
        assert_eq!(menu.selected_index, 0);
        assert_eq!(menu.items.len(), 3);
        assert_eq!(menu.items[0], "View Logs");
        assert_eq!(menu.items[1], "Open in Browser");
        assert_eq!(menu.items[2], "Close Menu");
    }

    #[test]
    fn test_context_menu_next() {
        let mut menu = ContextMenuComponent::new();
        
        menu.next();
        assert_eq!(menu.selected_index, 1);
        
        menu.next();
        assert_eq!(menu.selected_index, 2);
        
        menu.next();
        assert_eq!(menu.selected_index, 0); // Should wrap around
    }

    #[test]
    fn test_context_menu_previous() {
        let mut menu = ContextMenuComponent::new();
        
        menu.previous();
        assert_eq!(menu.selected_index, 2); // Should wrap to end
        
        menu.previous();
        assert_eq!(menu.selected_index, 1);
        
        menu.previous();
        assert_eq!(menu.selected_index, 0);
    }

    #[test]
    fn test_context_menu_get_selected_action() {
        let mut menu = ContextMenuComponent::new();
        
        assert_eq!(menu.get_selected_action(), "View Logs");
        
        menu.next();
        assert_eq!(menu.get_selected_action(), "Open in Browser");
        
        menu.next();
        assert_eq!(menu.get_selected_action(), "Close Menu");
    }

    #[test]
    fn test_context_menu_navigation_bounds() {
        let mut menu = ContextMenuComponent::new();
        
        // Test forward navigation bounds
        for _ in 0..10 {
            menu.next();
        }
        assert!(menu.selected_index < menu.items.len());
        
        // Test backward navigation bounds
        for _ in 0..10 {
            menu.previous();
        }
        assert!(menu.selected_index < menu.items.len());
    }

    #[test]
    fn test_context_menu_items_content() {
        let menu = ContextMenuComponent::new();
        
        // Verify all expected items are present
        let expected_items = vec![
            "View Logs",
            "Open in Browser", 
            "Close Menu"
        ];
        
        assert_eq!(menu.items, expected_items);
    }

    #[test]
    fn test_context_menu_initial_selection() {
        let menu = ContextMenuComponent::new();
        assert_eq!(menu.selected_index, 0);
        assert_eq!(menu.get_selected_action(), "View Logs");
    }

    #[test]
    fn test_context_menu_circular_navigation() {
        let mut menu = ContextMenuComponent::new();
        let initial_index = menu.selected_index;
        
        // Test complete forward cycle
        let steps = menu.items.len();
        for _ in 0..steps {
            menu.next();
        }
        assert_eq!(menu.selected_index, initial_index);
        
        // Test complete backward cycle
        for _ in 0..steps {
            menu.previous();
        }
        assert_eq!(menu.selected_index, initial_index);
    }
}