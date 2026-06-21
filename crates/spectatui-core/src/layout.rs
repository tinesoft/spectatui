use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaneKind {
    FeatureList,
    SpecBrowser,
    Constitution,
    ExtensionsPresets,
    WorkflowTimeline,
    AgentOutput,
    CliOutputLog,
}

impl PaneKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::FeatureList => "Features",
            Self::SpecBrowser => "Spec Browser",
            Self::Constitution => "Constitution",
            Self::ExtensionsPresets => "Extensions",
            Self::WorkflowTimeline => "Workflow",
            Self::AgentOutput => "Agent Output",
            Self::CliOutputLog => "CLI Output",
        }
    }

    pub fn min_height(&self) -> u16 {
        match self {
            Self::WorkflowTimeline => 11,
            Self::AgentOutput => 8,
            _ => 7,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaneConfig {
    pub kind: PaneKind,
    pub visible: bool,
    pub order: u8,
    pub size: u8,
}

impl PaneConfig {
    pub fn new(kind: PaneKind, order: u8) -> Self {
        Self {
            kind,
            visible: true,
            order,
            size: 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomLayout {
    pub panes: Vec<PaneConfig>,
}

impl Default for CustomLayout {
    fn default() -> Self {
        Self {
            panes: vec![
                PaneConfig::new(PaneKind::FeatureList, 0),
                PaneConfig::new(PaneKind::WorkflowTimeline, 1),
                PaneConfig::new(PaneKind::AgentOutput, 2),
                PaneConfig::new(PaneKind::SpecBrowser, 3),
                PaneConfig::new(PaneKind::Constitution, 4),
                PaneConfig {
                    kind: PaneKind::ExtensionsPresets,
                    visible: false,
                    order: 5,
                    size: 2,
                },
                PaneConfig {
                    kind: PaneKind::CliOutputLog,
                    visible: false,
                    order: 6,
                    size: 1,
                },
            ],
        }
    }
}

impl CustomLayout {
    pub fn visible_panes(&self) -> Vec<&PaneConfig> {
        let mut panes: Vec<&PaneConfig> = self.panes.iter().filter(|p| p.visible).collect();
        panes.sort_by_key(|p| p.order);
        panes
    }

    pub fn toggle_visibility(&mut self, kind: PaneKind) {
        if let Some(pane) = self.panes.iter_mut().find(|p| p.kind == kind) {
            pane.visible = !pane.visible;
        }
    }

    pub fn swap_order(&mut self, idx: usize, direction: i8) {
        let visible: Vec<usize> = self
            .panes
            .iter()
            .enumerate()
            .filter(|(_, p)| p.visible)
            .map(|(i, _)| i)
            .collect();

        if let Some(pos) = visible.iter().position(|&i| i == idx) {
            let new_pos = (pos as i64 + direction as i64).clamp(0, visible.len() as i64 - 1)
                as usize;
            if pos != new_pos {
                let a = visible[pos];
                let b = visible[new_pos];
                let ord_a = self.panes[a].order;
                let ord_b = self.panes[b].order;
                self.panes[a].order = ord_b;
                self.panes[b].order = ord_a;
            }
        }
    }

    pub fn resize_pane(&mut self, kind: PaneKind, delta: i8) {
        if let Some(pane) = self.panes.iter_mut().find(|p| p.kind == kind) {
            pane.size = (pane.size as i8 + delta).clamp(1, 4) as u8;
        }
    }
}
