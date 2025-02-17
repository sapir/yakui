use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use glam::Vec2;
use smallvec::SmallVec;

use crate::dom::{Dom, DomNode};
use crate::event::{EventInterest, EventResponse, WidgetEvent};
use crate::id::WidgetId;
use crate::layout::LayoutDom;

use super::button::MouseButton;
use super::KeyboardKey;

/// Holds yakui's input state, like cursor position, hovered, and selected
/// widgets.
#[derive(Debug)]
pub struct InputState {
    inner: Rc<InputStateInner>,
}

impl InputState {
    /// Create a new, empty `InputState`.
    pub fn new() -> Self {
        Self {
            inner: Rc::new(InputStateInner::new()),
        }
    }

    pub(crate) fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }

    /// Return the currently selected widget, if there is one.
    pub fn selection(&self) -> Option<WidgetId> {
        *self.inner.selection.borrow()
    }

    /// Set the currently selected widget.
    pub fn set_selection(&self, id: Option<WidgetId>) {
        let mut selection = self.inner.selection.borrow_mut();
        *selection = id;
    }

    pub(crate) fn mouse_moved(&self, dom: &Dom, layout: &LayoutDom, pos: Option<Vec2>) {
        self.inner.mouse_moved(dom, layout, pos);
    }

    pub(crate) fn mouse_button_changed(
        &self,
        dom: &Dom,
        layout: &LayoutDom,
        button: MouseButton,
        down: bool,
    ) -> EventResponse {
        self.inner.mouse_button_changed(dom, layout, button, down)
    }

    pub(crate) fn keyboard_key_changed(
        &self,
        dom: &Dom,
        layout: &LayoutDom,
        key: KeyboardKey,
        down: bool,
    ) -> EventResponse {
        self.inner.keyboard_key_changed(dom, layout, key, down)
    }

    pub(crate) fn text_input(&self, dom: &Dom, layout: &LayoutDom, c: char) -> EventResponse {
        self.inner.text_input(dom, layout, c)
    }

    pub(crate) fn finish(&self) {
        self.inner.finish()
    }
}

#[derive(Debug)]
struct InputStateInner {
    /// State for the mouse, like buttons and position.
    mouse: RefCell<Mouse>,

    /// Details about widgets and their mouse intersections.
    intersections: RefCell<Intersections>,

    /// The widget that is currently selected.
    selection: RefCell<Option<WidgetId>>,
}

#[derive(Debug)]
struct Mouse {
    /// The current mouse position, or `None` if it's outside the window.
    position: Option<Vec2>,

    /// The state of each mouse button. If missing from the map, the button is
    /// up and has not yet been pressed.
    buttons: HashMap<MouseButton, ButtonState>,
}

#[derive(Debug)]
struct Intersections {
    /// All of the widgets with mouse interest that the current mouse position
    /// intersects with.
    ///
    /// All lists like this are stored in reverse depth first order.
    mouse_hit: Vec<WidgetId>,

    /// All of the widgets that have had a mouse enter event sent to them
    /// without a corresponding mouse leave event yet. This is different from
    /// mouse_hit because hover events can be sunk by event handlers.
    mouse_entered: Vec<WidgetId>,

    /// All of the widgets that had a mouse enter event sent to them and then
    /// sunk it that are still being hovered. This helps us ensure that a widget
    /// that sunk a hover event will continue to occupy that space even if we
    /// don't send it more events.
    mouse_entered_and_sunk: Vec<WidgetId>,

    /// All widgets that had the corresponding mouse button pressed while the
    /// mouse cursor was over them.
    #[allow(unused)]
    mouse_down_in: HashMap<MouseButton, Vec<WidgetId>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ButtonState {
    JustDown,
    Down,
    JustUp,
    Up,
}

impl ButtonState {
    pub fn is_down(&self) -> bool {
        matches!(self, Self::JustDown | Self::Down)
    }

    pub fn settle(&mut self) {
        match self {
            Self::JustDown => {
                *self = Self::Down;
            }
            Self::JustUp => {
                *self = Self::Up;
            }
            _ => (),
        }
    }
}

impl InputStateInner {
    fn new() -> Self {
        Self {
            mouse: RefCell::new(Mouse {
                position: None,
                buttons: HashMap::new(),
            }),
            intersections: RefCell::new(Intersections {
                mouse_hit: Vec::new(),
                mouse_entered: Vec::new(),
                mouse_entered_and_sunk: Vec::new(),
                mouse_down_in: HashMap::new(),
            }),
            selection: RefCell::new(None),
        }
    }

    /// Signal that the mouse has moved.
    fn mouse_moved(&self, dom: &Dom, layout: &LayoutDom, pos: Option<Vec2>) {
        {
            let mut mouse = self.mouse.borrow_mut();
            mouse.position = pos;
        }

        self.mouse_hit_test(dom, layout);
        self.send_mouse_enter(dom);
        self.send_mouse_leave(dom);
    }

    /// Signal that a mouse button's state has changed.
    fn mouse_button_changed(
        &self,
        dom: &Dom,
        layout: &LayoutDom,
        button: MouseButton,
        down: bool,
    ) -> EventResponse {
        {
            let mut mouse = self.mouse.borrow_mut();
            let state = mouse.buttons.entry(button).or_insert(ButtonState::Up);

            match (state.is_down(), down) {
                // If the state didn't actually change, leave the current value
                // alone.
                (true, true) | (false, false) => (),

                (false, true) => {
                    *state = ButtonState::JustDown;
                }

                (true, false) => {
                    *state = ButtonState::JustUp;
                }
            }
        }

        self.send_button_change(dom, layout, button, down)
    }

    fn keyboard_key_changed(
        &self,
        dom: &Dom,
        layout: &LayoutDom,
        key: KeyboardKey,
        down: bool,
    ) -> EventResponse {
        let selected = *self.selection.borrow();
        if let Some(id) = selected {
            let layout_node = layout.get(id).unwrap();

            if layout_node
                .event_interest
                .contains(EventInterest::FOCUSED_KEYBOARD)
            {
                let mut node = dom.get_mut(id).unwrap();
                let event = WidgetEvent::KeyChanged(key, down);
                return fire_event(dom, id, &mut node, &event);
            }
        }

        EventResponse::Bubble
    }

    fn text_input(&self, dom: &Dom, layout: &LayoutDom, c: char) -> EventResponse {
        let selected = *self.selection.borrow();
        if let Some(id) = selected {
            let layout_node = layout.get(id).unwrap();

            if layout_node
                .event_interest
                .contains(EventInterest::FOCUSED_KEYBOARD)
            {
                let mut node = dom.get_mut(id).unwrap();
                let event = WidgetEvent::TextInput(c);
                return fire_event(dom, id, &mut node, &event);
            }
        }

        EventResponse::Bubble
    }

    /// Finish applying input events for this frame.
    fn finish(&self) {
        self.settle_buttons();
    }

    fn send_button_change(
        &self,
        dom: &Dom,
        layout: &LayoutDom,
        button: MouseButton,
        value: bool,
    ) -> EventResponse {
        let intersections = self.intersections.borrow();
        let mut overall_response = EventResponse::Bubble;

        for &id in &intersections.mouse_hit {
            if let Some(mut node) = dom.get_mut(id) {
                let event = WidgetEvent::MouseButtonChanged(button, value);
                let response = fire_event(dom, id, &mut node, &event);

                if response == EventResponse::Sink {
                    overall_response = response;
                    break;
                }
            }
        }

        // For consistency, reverse the interest_mouse array like we do in
        // hit_test. This event can't be sunk, so it's not super important.
        let interest_mouse = layout.interest_mouse.iter().copied().rev();

        for (id, interest) in interest_mouse {
            if interest.contains(EventInterest::MOUSE_OUTSIDE)
                && !intersections.mouse_hit.contains(&id)
            {
                if let Some(mut node) = dom.get_mut(id) {
                    let event = WidgetEvent::MouseButtonChangedOutside(button, value);
                    fire_event(dom, id, &mut node, &event);
                }
            }
        }

        overall_response
    }

    fn send_mouse_enter(&self, dom: &Dom) {
        let mut intersections = self.intersections.borrow_mut();
        let intersections = &mut *intersections;

        for &hit in &intersections.mouse_hit {
            if let Some(mut node) = dom.get_mut(hit) {
                if !intersections.mouse_entered.contains(&hit) {
                    intersections.mouse_entered.push(hit);

                    let response = fire_event(dom, hit, &mut node, &WidgetEvent::MouseEnter);

                    if response == EventResponse::Sink {
                        intersections.mouse_entered_and_sunk.push(hit);
                        break;
                    }
                } else if intersections.mouse_entered_and_sunk.contains(&hit) {
                    // This widget was hovered previously, is still hovered, and
                    // sunk the mouse enter event before. In order to prevent
                    // erroneously hovering other widgets, continue sinking this
                    // event.
                    break;
                }
            }
        }
    }

    fn send_mouse_leave(&self, dom: &Dom) {
        let mut intersections = self.intersections.borrow_mut();

        let mut to_remove = SmallVec::<[WidgetId; 4]>::new();

        for &hit in &intersections.mouse_entered {
            if !intersections.mouse_hit.contains(&hit) {
                if let Some(mut node) = dom.get_mut(hit) {
                    fire_event(dom, hit, &mut node, &WidgetEvent::MouseLeave);
                }

                to_remove.push(hit);
            }
        }

        for remove in to_remove {
            intersections.mouse_entered.retain(|&id| id != remove);
            intersections
                .mouse_entered_and_sunk
                .retain(|&id| id != remove);
        }
    }

    fn mouse_hit_test(&self, dom: &Dom, layout: &LayoutDom) {
        let mut intersections = self.intersections.borrow_mut();
        let mouse = self.mouse.borrow();

        intersections.mouse_hit.clear();

        if let Some(mut mouse_pos) = mouse.position {
            mouse_pos /= layout.scale_factor();
            hit_test(dom, layout, mouse_pos, &mut intersections.mouse_hit);
        }
    }

    fn settle_buttons(&self) {
        let mut mouse = self.mouse.borrow_mut();

        for state in mouse.buttons.values_mut() {
            state.settle();
        }
    }
}

/// Notify the widget of an event, pushing it onto the stack first to ensure
/// that the DOM will have the correct widget at the top of the stack if
/// queried.
fn fire_event(dom: &Dom, id: WidgetId, node: &mut DomNode, event: &WidgetEvent) -> EventResponse {
    dom.enter(id);
    let response = node.widget.event(event);
    dom.exit(id);

    response
}

#[profiling::function]
fn hit_test(_dom: &Dom, layout: &LayoutDom, coords: Vec2, output: &mut Vec<WidgetId>) {
    // interest_mouse is stored in layout traversal order, which is depth first.
    //
    // We want to test against the deepest widgets in the tree first and bubble
    // to their ancestors first.
    let interest_mouse = layout.interest_mouse.iter().copied().rev();

    for (id, _interest) in interest_mouse {
        let layout_node = layout.get(id).unwrap();

        if layout_node.rect.contains_point(coords) {
            output.push(id);
        }
    }
}
