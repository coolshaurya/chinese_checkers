use coffee::graphics::Point;
use coffee::input::{mouse, ButtonState, Event, Input};

#[derive(Debug, Clone, Default)]
pub struct DragNDrop {
    has_drag_started: bool,
    start_drag_pos: Option<Point>,
    current_drag_pos: Option<Point>,
    is_dropped: bool,
}

impl Input for DragNDrop {
    fn new() -> Self {
        Self::default()
    }

    fn update(&mut self, event: Event) {
        // we only care for mouse events
        if let Event::Mouse(event) = event {
            if let mouse::Event::CursorMoved { x, y } = event {
                let point = [x, y].into();
                match (
                    self.has_drag_started,
                    self.start_drag_pos.is_some(),
                    self.current_drag_pos.is_some(),
                ) {
                    // drag not started, do nothing
                    (false, _, _) => {}
                    // drag started but start postition not recorded
                    (true, false, false) => {
                        self.start_drag_pos = Some(point);
                    }
                    // drag started and start position recorded
                    (true, true, _) => {
                        if nalgebra::distance(&self.start_drag_pos.unwrap(), &point) > 5.0 {
                            self.current_drag_pos = Some(point);
                        }
                    }
                    // all other combinations are invalid
                    _ => unreachable!(),
                }
            } else if let mouse::Event::Input { button, state } = event {
                // we only care about the left mouse button
                if let mouse::Button::Left = button {
                    match state {
                        ButtonState::Pressed => {
                            self.has_drag_started = true;
                        }
                        ButtonState::Released => {
                            if self.current_drag_pos.is_none() {
                                self.reset()
                            } else {
                                self.is_dropped = true;
                            }
                        }
                    }
                }
            }
        }
    }

    fn clear(&mut self) {
        if self.is_dropped {
            self.reset();
        }
    }
}

impl DragNDrop {
    fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn is_dropped(&self) -> bool {
        self.is_dropped
    }
    pub fn drag_status(&self) -> Option<(Point, Point)> {
        Some((self.current_drag_pos?, self.start_drag_pos?))
    }
}
