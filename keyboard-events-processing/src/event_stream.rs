pub use core_graphics::event::CGKeyCode;
pub use core_graphics::event::KeyCode;

pub trait KeyboardEvent: Sized {
    fn keycode(&self) -> CGKeyCode;
    fn set_keycode(&self, keycode: CGKeyCode);

    fn update_keycode<F: FnOnce(CGKeyCode) -> CGKeyCode>(&self, update_callback: F) {
        let keycode = self.keycode();
        self.set_keycode(update_callback(keycode));
    }
}

pub trait EventStream<E> {
    type Decision;

    fn post(&mut self, event: &E);
    fn pass_current_event(self) -> Self::Decision;
    fn drop_current_event(self) -> Self::Decision;
    fn steal_current_event<F: FnOnce(E)>(self, steal_callback: F) -> Self::Decision;
}

#[derive(Clone, Copy, Debug)]
pub enum EventType {
    KeyDown,
    KeyUp,
}

pub trait EventProcessor<E> {
    #[must_use]
    fn process<'event, S: EventStream<E> + 'event>(
        &mut self,
        timestamp: std::time::Instant,
        event_type: EventType,
        event: &'event E,
        stream: S,
    ) -> S::Decision;
}
