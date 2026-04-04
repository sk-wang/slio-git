//! Generic collapsible tree widget for branch/file display

use crate::theme;
use iced::widget::{button, Column, Container, Row, Space, Text};
use iced::{Alignment, Element, Length, Point};

/// A node in the tree
#[derive(Debug, Clone)]
pub struct TreeNode {
    /// Unique identifier
    pub id: String,
    /// Display label
    pub label: String,
    /// Child nodes
    pub children: Vec<TreeNode>,
    /// Whether this node is expanded
    pub expanded: bool,
    /// Optional icon text (emoji or symbol)
    pub icon: Option<String>,
    /// Whether this node is a leaf (no children possible)
    pub is_leaf: bool,
    /// Indent depth (computed during rendering)
    pub depth: u32,
}

impl TreeNode {
    /// Create a new leaf node
    pub fn leaf(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            children: Vec::new(),
            expanded: false,
            icon: None,
            is_leaf: true,
            depth: 0,
        }
    }

    /// Create a new group node
    pub fn group(
        id: impl Into<String>,
        label: impl Into<String>,
        children: Vec<TreeNode>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            children,
            expanded: true,
            icon: None,
            is_leaf: false,
            depth: 0,
        }
    }

    /// Set icon
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set expanded state
    pub fn with_expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }
}

/// Messages emitted by the tree widget
#[derive(Debug, Clone)]
pub enum TreeMessage {
    /// A node was clicked
    SelectNode(String),
    /// A node was expanded/collapsed
    ToggleNode(String),
    /// A node was right-clicked at a position
    NodeContextMenu(String, Point),
}

/// Render a tree widget
pub fn tree_view<'a, Message: Clone + 'a>(
    nodes: &'a [TreeNode],
    selected_id: Option<&'a str>,
    search_filter: Option<&'a str>,
    on_message: impl Fn(TreeMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let mut column = Column::new().spacing(0);

    for node in nodes {
        if let Some(filter) = search_filter {
            if !node_matches_filter(node, filter) {
                continue;
            }
        }
        column = render_node(column, node, 0, selected_id, search_filter, &on_message);
    }

    Container::new(column).width(Length::Fill).into()
}

fn node_matches_filter(node: &TreeNode, filter: &str) -> bool {
    let filter_lower = filter.to_lowercase();
    if node.label.to_lowercase().contains(&filter_lower) {
        return true;
    }
    node.children.iter().any(|c| node_matches_filter(c, filter))
}

fn render_node<'a, Message: Clone + 'a>(
    mut column: Column<'a, Message>,
    node: &'a TreeNode,
    depth: u32,
    selected_id: Option<&'a str>,
    search_filter: Option<&'a str>,
    on_message: &(impl Fn(TreeMessage) -> Message + Clone + 'a),
) -> Column<'a, Message> {
    let is_selected = selected_id == Some(node.id.as_str());
    let indent = depth as f32 * 16.0;

    let expand_icon = if !node.is_leaf {
        if node.expanded {
            "▼"
        } else {
            "▶"
        }
    } else {
        "  "
    };

    let node_id = node.id.clone();
    let on_msg = on_message.clone();

    let mut row = Row::new()
        .spacing(4)
        .align_y(Alignment::Center)
        .push(Space::new().width(Length::Fixed(indent)));

    if !node.is_leaf {
        let toggle_id = node.id.clone();
        let on_msg_toggle = on_message.clone();
        row = row.push(
            button(Text::new(expand_icon).size(10).color(theme::darcula::TEXT_SECONDARY))
                .style(|_, _| button::Style::default())
                .padding(0)
                .on_press(on_msg_toggle(TreeMessage::ToggleNode(toggle_id))),
        );
    } else {
        row = row.push(Space::new().width(Length::Fixed(14.0)));
    }

    if let Some(icon) = &node.icon {
        row = row.push(Text::new(icon.as_str()).size(12));
    }

    let label_color = if is_selected {
        theme::darcula::TEXT_PRIMARY
    } else {
        theme::darcula::TEXT_SECONDARY
    };

    row = row.push(Text::new(node.label.as_str()).size(12).color(label_color));

    if !node.is_leaf {
        let count = count_leaves(node);
        row = row.push(
            Text::new(format!(" ({})", count))
                .size(10)
                .color(theme::darcula::TEXT_DISABLED),
        );
    }

    let row_button = button(row)
        .style(|_, _| button::Style::default())
        .padding([2, 4])
        .on_press(on_msg(TreeMessage::SelectNode(node_id)));

    let row_container = if is_selected {
        Container::new(row_button)
            .width(Length::Fill)
            .style(theme::panel_style(theme::Surface::Raised))
    } else {
        Container::new(row_button).width(Length::Fill)
    };

    column = column.push(row_container);

    // Render children if expanded
    if !node.is_leaf && node.expanded {
        for child in &node.children {
            if let Some(filter) = search_filter {
                if !node_matches_filter(child, filter) {
                    continue;
                }
            }
            column = render_node(column, child, depth + 1, selected_id, search_filter, on_message);
        }
    }

    column
}

fn count_leaves(node: &TreeNode) -> usize {
    if node.is_leaf {
        return 1;
    }
    node.children.iter().map(count_leaves).sum()
}
