#![allow(dead_code)]

use std::{fmt::Debug, hash::Hash};

use eframe::egui::{Align, Align2, Id, Layout, Pos2, Rect, Response, Sense, Ui};

// Can it be dragged with mouse? If set to Prioritized, it will prioritize
// mouse drag over setting position with .pos(), however, when you let
// go of it, it will set position again using .pos() method
#[derive(PartialEq, Debug, Clone)]
pub enum Dragging {
    Disabled,
    Enabled,
    Prioritized,
}

// This is like an Area, but it's inside another container and uses position
// relative to its parent. Also has some additional features
#[derive(Clone, Debug)]
#[must_use = "You should call .show_inside()"]
pub struct RelArea {
    id: Id,
    align: Option<Align2>,
    // Should be None if align is None
    offset: Option<Pos2>,
    max_width: Option<f32>,
    max_height: Option<f32>,
    dragging: Dragging,
    enabled: bool,
    default_pos: Pos2,
    // Set with .pos method
    new_pos: Option<Pos2>,
    ignore_bounds: bool,
}

impl RelArea {
    pub fn new(id_source: impl Hash) -> Self {
        let id = Id::new(id_source);
        Self {
            id,
            align: None,
            offset: None,
            max_width: None,
            max_height: None,
            dragging: Dragging::Enabled,
            enabled: true,
            default_pos: Pos2 { x: 0.0, y: 0.0 },
            new_pos: None,
            ignore_bounds: false,
        }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn dragging(&self) -> bool {
        self.dragging != Dragging::Disabled && self.enabled
    }

    // Tells it how to process mouse dragging. See Dragging enum
    pub fn set_dragging(mut self, dragging: Dragging) -> Self {
        self.dragging = dragging;
        self
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// If false, all widgets inside will be disabled and grayed out
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Has lower priority than setting position with .pos method.
    /// Disables ability to drag the area
    pub fn align(mut self, align: Align2) -> Self {
        self.align = Some(align);
        self.dragging = Dragging::Disabled;
        self
    }

    pub fn offset(mut self, offset: impl Into<Pos2>) -> Self {
        self.offset = Some(offset.into());
        self
    }

    pub fn max_width(mut self, max_width: f32) -> Self {
        self.max_width = Some(max_width);
        self
    }

    pub fn max_height(mut self, max_height: f32) -> Self {
        self.max_width = Some(max_height);
        self
    }

    /// Has higher priority than setting position with .align(), but with mouse
    /// drag it's a bit more complicated. See docs for Dragging enum
    pub fn set_pos(mut self, pos: impl Into<Pos2>) -> Self {
        self.new_pos = Some(pos.into());
        self
    }

    /// Initial position. Then it can be changed by mouse drag or .pos() method
    pub fn default_pos(mut self, default_pos: impl Into<Pos2>) -> Self {
        self.default_pos = default_pos.into();
        self
    }

    pub fn ignore_bounds(mut self) -> Self {
        self.ignore_bounds = true;
        self
    }
}

impl RelArea {
    /// This is not like other .show_inside methods, this one also returns
    /// RelArea's relative position in a tuple
    pub fn show_inside<R>(
        self,
        parent_ui: &mut Ui,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> RelAreaResponse<R> {
        // Loading last known position
        let mut pos = parent_ui
            .ctx()
            .memory()
            .data
            .get_temp::<Pos2>(self.id)
            .unwrap_or(self.default_pos);

        // Drawing the Ui
        let max_draw_rect = Rect {
            min: parent_ui.max_rect().min + pos.to_vec2(),
            max: Pos2 {
                x: match self.max_width {
                    Some(max_width) => parent_ui.max_rect().min.x + max_width,
                    None => parent_ui.max_rect().max.x,
                },
                y: match self.max_height {
                    Some(max_height) => parent_ui.max_rect().min.y + max_height,
                    None => parent_ui.max_rect().max.y,
                },
            },
        };
        let mut ui = parent_ui.child_ui(max_draw_rect, Layout::default());
        ui.set_enabled(self.enabled);
        let inner_response = add_contents(&mut ui);

        // Aligning the area
        let (width, height) = (ui.min_rect().width(), ui.min_rect().height());
        if let Some(align) = self.align {
            pos.x = match align.x() {
                Align::Min => 0.0,
                Align::Center => (parent_ui.max_rect().width() - width) / 2.0,
                Align::Max => parent_ui.max_rect().width() - width,
            };
            pos.y = match align.y() {
                Align::Min => 0.0,
                Align::Center => (parent_ui.max_rect().height() - height) / 2.0,
                Align::Max => parent_ui.max_rect().height() - height,
            };
            if let Some(offset) = self.offset {
                pos += offset.to_vec2();
            }
        }

        // Processing events and dragging
        let sense = match self.dragging {
            Dragging::Disabled => Sense::click(),
            _ => Sense::click_and_drag(),
        };
        let response = ui.interact(ui.min_rect(), self.id, sense);
        if response.dragged() {
            match self.dragging {
                Dragging::Enabled => {
                    pos += parent_ui.ctx().input().pointer.delta();
                    pos = self.new_pos.unwrap_or(pos);
                }
                Dragging::Prioritized => pos += parent_ui.ctx().input().pointer.delta(),
                _ => (),
            }
        } else {
            pos = self.new_pos.unwrap_or(pos);
        }

        // Constraining position to its bounds
        if !self.ignore_bounds {
            let max_x = parent_ui.max_rect().width() - width;
            let max_y = parent_ui.max_rect().height() - height;
            if max_x > 0.0 {
                pos.x = (pos.x).clamp(0.0, max_x);
            }
            if max_y > 0.0 {
                pos.y = (pos.y).clamp(0.0, max_y)
            }
        }

        // Saving (possibly) new position
        parent_ui.ctx().memory().data.insert_temp(self.id, pos);

        RelAreaResponse {
            inner_response,
            response,
            current_pos: pos,
        }
    }
}

pub struct RelAreaResponse<T> {
    pub response: Response,
    pub inner_response: T,
    pub current_pos: Pos2,
}
