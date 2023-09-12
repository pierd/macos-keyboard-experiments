use keyboard_events_processing::event_stream::{EventProcessor, EventStream, EventType};

pub struct StatsProcessor {
    last_tap: std::time::Instant,
}

impl Default for StatsProcessor {
    fn default() -> Self {
        Self {
            last_tap: std::time::Instant::now(),
        }
    }
}

impl<E> EventProcessor<E> for StatsProcessor {
    fn process<'e, S: EventStream<E> + 'e>(
        &mut self,
        timestamp: std::time::Instant,
        event_type: EventType,
        _event: &'e E,
        stream: S,
    ) -> <S as EventStream<E>>::Decision {
        match event_type {
            EventType::KeyDown => {
                println!("Duration: {:?}", timestamp.duration_since(self.last_tap));
                self.last_tap = timestamp;
            }
            EventType::KeyUp => {
                println!("Pressed for: {:?}", timestamp.duration_since(self.last_tap));
            }
        }
        stream.pass_current_event()
    }
}
