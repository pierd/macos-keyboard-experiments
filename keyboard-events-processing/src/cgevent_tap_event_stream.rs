use core_graphics::event::CGEvent;
use libc::c_void;

use crate::{
    event_stream::{EventStream, KeyboardEvent},
    event_tap::CGEventTapActiveFilterDecision,
};

impl KeyboardEvent for CGEvent {
    fn keycode(&self) -> crate::event_stream::CGKeyCode {
        self.get_integer_value_field(core_graphics::event::EventField::KEYBOARD_EVENT_KEYCODE)
            as crate::event_stream::CGKeyCode
    }

    fn set_keycode(&self, keycode: crate::event_stream::CGKeyCode) {
        self.set_integer_value_field(
            core_graphics::event::EventField::KEYBOARD_EVENT_KEYCODE,
            keycode as i64,
        );
    }
}

pub struct CGEventStream<'processing> {
    tap_proxy: *const c_void,
    current_event: &'processing CGEvent,
}

impl<'a> CGEventStream<'a> {
    pub fn new(tap_proxy: *const c_void, current_event: &'a CGEvent) -> Self {
        Self {
            tap_proxy,
            current_event,
        }
    }
}

impl<'a> EventStream<CGEvent> for CGEventStream<'a> {
    type Decision = CGEventTapActiveFilterDecision;

    fn post(&mut self, event: &CGEvent) {
        event.post_from_tap(self.tap_proxy)
    }

    fn pass_current_event(self) -> Self::Decision {
        CGEventTapActiveFilterDecision::Pass
    }

    fn drop_current_event(self) -> Self::Decision {
        CGEventTapActiveFilterDecision::Drop
    }

    fn steal_current_event<F: FnOnce(CGEvent)>(self, steal_callback: F) -> Self::Decision {
        steal_callback(self.current_event.clone());
        CGEventTapActiveFilterDecision::Drop
    }
}
