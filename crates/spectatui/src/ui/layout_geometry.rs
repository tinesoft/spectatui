use ratatui::layout::{Constraint, Layout, Rect};
use spectatui_core::layout::{CustomLayout, DashboardSizes, PaneConfig, PaneKind};

use crate::app::{App, DividerAxis, DividerTarget, ResizeDivider};

pub(super) fn custom_layout_geometry(
    custom_layout: &CustomLayout,
    sidebar_size: u16,
    runtime_sizes: &DashboardSizes,
    area: Rect,
) -> (Vec<(PaneKind, Rect)>, Vec<ResizeDivider>) {
    let visible = custom_layout.visible_panes();
    if visible.is_empty() {
        return (Vec::new(), Vec::new());
    }
    if visible.len() == 1 {
        return (vec![(visible[0].kind, area)], Vec::new());
    }

    let first = visible[0].kind;
    let rest = &visible[1..];
    let min_rest_width = rest
        .iter()
        .map(|pane| min_width(pane.kind))
        .max()
        .unwrap_or(1);
    let cols = split_horizontal(
        area,
        stored_or_length(sidebar_size, 38, area.width),
        min_width(first),
        min_rest_width,
    );
    let mut out = vec![(visible[0].kind, cols[0])];
    let mut dividers = vec![vertical_divider(
        DividerTarget::CustomSidebar,
        area,
        cols[0],
        min_width(first),
        min_rest_width,
    )];
    let right_rows = weighted_rows(rest, runtime_sizes, cols[1]);
    for (i, (pane, rect)) in rest.iter().zip(right_rows.iter()).enumerate() {
        out.push((pane.kind, *rect));
        if let Some(next) = rest.get(i + 1) {
            let next_rect = right_rows[i + 1];
            let bounds = Rect::new(
                cols[1].x,
                rect.y,
                cols[1].width,
                rect.height.saturating_add(next_rect.height),
            );
            dividers.push(horizontal_divider(
                DividerTarget::CustomStack(pane.kind, next.kind),
                bounds,
                *rect,
                pane.kind.min_height(),
                next.kind.min_height(),
            ));
        }
    }
    (out, dividers)
}

pub(super) fn min_width(kind: PaneKind) -> u16 {
    match kind {
        PaneKind::FeatureList => 24,
        PaneKind::SpecBrowser | PaneKind::Constitution => 28,
        PaneKind::ExtensionsPresets => 26,
        _ => 20,
    }
}

pub(super) fn stored_or_default(stored: u16, default: u16) -> u16 {
    if stored == 0 {
        default
    } else {
        stored
    }
}

pub(super) fn stored_or_length(stored: u16, default_length: u16, total: u16) -> u16 {
    if stored == 0 || total == 0 {
        (default_length as u32 * 10_000 / total.max(1) as u32).min(10_000) as u16
    } else {
        stored
    }
}

pub(super) fn split_horizontal(
    area: Rect,
    percent: u16,
    min_left: u16,
    min_right: u16,
) -> [Rect; 2] {
    let left_width = split_length(area.width, percent, min_left, min_right);
    [
        Rect::new(area.x, area.y, left_width, area.height),
        Rect::new(
            area.x.saturating_add(left_width),
            area.y,
            area.width.saturating_sub(left_width),
            area.height,
        ),
    ]
}

pub(super) fn split_vertical(area: Rect, percent: u16, min_top: u16, min_bottom: u16) -> [Rect; 2] {
    let top_height = split_length(area.height, percent, min_top, min_bottom);
    [
        Rect::new(area.x, area.y, area.width, top_height),
        Rect::new(
            area.x,
            area.y.saturating_add(top_height),
            area.width,
            area.height.saturating_sub(top_height),
        ),
    ]
}

fn split_length(total: u16, percent: u16, min_before: u16, min_after: u16) -> u16 {
    let upper = total.saturating_sub(min_after);
    let lower = min_before.min(upper);
    (((total as u32 * percent.min(10_000) as u32) / 10_000) as u16).clamp(lower, upper)
}

fn weighted_rows(panes: &[&PaneConfig], runtime_sizes: &DashboardSizes, area: Rect) -> Vec<Rect> {
    let total_weight: u32 = panes
        .iter()
        .map(|pane| runtime_sizes.custom_pane_height(pane.kind, pane.size) as u32)
        .sum::<u32>()
        .max(1);
    let constraints: Vec<Constraint> = panes
        .iter()
        .map(|pane| {
            Constraint::Ratio(
                runtime_sizes.custom_pane_height(pane.kind, pane.size) as u32,
                total_weight,
            )
        })
        .collect();
    Layout::vertical(constraints).split(area).to_vec()
}

pub(super) fn register_vertical_divider(
    app: &App,
    target: DividerTarget,
    bounds: Rect,
    before: Rect,
    min_before: u16,
    min_after: u16,
) {
    app.register_resize_divider(vertical_divider(
        target, bounds, before, min_before, min_after,
    ));
}

fn vertical_divider(
    target: DividerTarget,
    bounds: Rect,
    before: Rect,
    min_before: u16,
    min_after: u16,
) -> ResizeDivider {
    ResizeDivider {
        target,
        axis: DividerAxis::Vertical,
        bounds,
        min_before,
        min_after,
        hitbox: Rect::new(
            before.x.saturating_add(before.width.saturating_sub(1)),
            bounds.y,
            2,
            bounds.height,
        ),
    }
}

pub(super) fn register_horizontal_divider(
    app: &App,
    target: DividerTarget,
    bounds: Rect,
    before: Rect,
    min_before: u16,
    min_after: u16,
) {
    app.register_resize_divider(horizontal_divider(
        target, bounds, before, min_before, min_after,
    ));
}

fn horizontal_divider(
    target: DividerTarget,
    bounds: Rect,
    before: Rect,
    min_before: u16,
    min_after: u16,
) -> ResizeDivider {
    ResizeDivider {
        target,
        axis: DividerAxis::Horizontal,
        bounds,
        min_before,
        min_after,
        hitbox: Rect::new(
            bounds.x,
            before.y.saturating_add(before.height.saturating_sub(1)),
            bounds.width,
            2,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_length_honors_both_minimums() {
        assert_eq!(split_length(100, 500, 30, 40), 30);
        assert_eq!(split_length(100, 9_500, 30, 40), 60);
    }

    #[test]
    fn stored_length_uses_the_legacy_default_until_dragged() {
        assert_eq!(stored_or_length(0, 38, 100), 3_800);
        assert_eq!(stored_or_length(6_200, 38, 100), 6_200);
    }
}
