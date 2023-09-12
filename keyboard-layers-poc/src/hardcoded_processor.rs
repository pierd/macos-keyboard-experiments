use keyboard_events_processing::{
    event_stream::{EventProcessor, EventStream, EventType, KeyCode, KeyboardEvent},
    keycodes::AnsiKeyCode,
};

const TAPPING_TERM: std::time::Duration = std::time::Duration::from_millis(200);

/// Creates a layer: j + esdf => arrow keys
pub struct HardcodedProcessor<E> {
    layer_pressed_time: Option<std::time::Instant>,
    stolen_events: Vec<E>,
}

impl<E> Default for HardcodedProcessor<E> {
    fn default() -> Self {
        Self {
            layer_pressed_time: None,
            stolen_events: Vec::new(),
        }
    }
}

impl<E: KeyboardEvent> EventProcessor<E> for HardcodedProcessor<E> {
    fn process<'e, S: EventStream<E> + 'e>(
        &mut self,
        timestamp: std::time::Instant,
        event_type: EventType,
        event: &'e E,
        mut stream: S,
    ) -> <S as EventStream<E>>::Decision {
        let keycode = event.keycode();
        if keycode == AnsiKeyCode::J {
            // layer key handling
            match event_type {
                EventType::KeyDown => {
                    if self.layer_pressed_time.is_none() {
                        self.layer_pressed_time = Some(timestamp);
                        stream.steal_current_event(|stolen_event| {
                            self.stolen_events.push(stolen_event)
                        })
                    } else {
                        eprintln!("Layer already pressed");
                        stream.pass_current_event()
                    }
                }
                EventType::KeyUp => {
                    if let Some(pressed_instant) = self.layer_pressed_time.take() {
                        if timestamp.duration_since(pressed_instant) < TAPPING_TERM {
                            // too quick - make it into a tap
                            for event in self.stolen_events.drain(..) {
                                stream.post(&event);
                            }
                            stream.pass_current_event()
                        } else {
                            // long enough - it's a hold
                            stream.drop_current_event()
                        }
                    } else {
                        // layer was cancelled
                        for event in self.stolen_events.drain(..) {
                            stream.post(&event);
                        }
                        stream.pass_current_event()
                    }
                }
            }
        } else if let Some(layer_pressed_instant) = self.layer_pressed_time.as_ref() {
            // in layer
            if [
                AnsiKeyCode::S,
                AnsiKeyCode::D,
                AnsiKeyCode::F,
                AnsiKeyCode::E,
            ]
            .contains(&keycode)
            {
                if timestamp.duration_since(*layer_pressed_instant) < TAPPING_TERM {
                    // quick - this might still be an error (ABAB) - steal events and wait for the keyup
                    match event_type {
                        EventType::KeyDown => stream.steal_current_event(|stolen_event| {
                            self.stolen_events.push(stolen_event)
                        }),
                        EventType::KeyUp => {
                            if self.stolen_events.last().unwrap().keycode() == keycode {
                                let down_event = self.stolen_events.pop().unwrap();
                                let new_keycode = match keycode {
                                    AnsiKeyCode::S => KeyCode::LEFT_ARROW,
                                    AnsiKeyCode::D => KeyCode::DOWN_ARROW,
                                    AnsiKeyCode::F => KeyCode::RIGHT_ARROW,
                                    AnsiKeyCode::E => KeyCode::UP_ARROW,
                                    _ => unreachable!(),
                                };
                                down_event.set_keycode(new_keycode);
                                stream.post(&down_event);
                                event.set_keycode(new_keycode);
                                stream.pass_current_event()
                            } else {
                                // mismatch - cancel
                                self.layer_pressed_time = None;
                                for event in self.stolen_events.drain(..) {
                                    stream.post(&event);
                                }
                                stream.pass_current_event()
                            }
                        }
                    }
                } else {
                    // long - this is a hold so we can process the events immediately
                    // first replay any stolen events (apart from the initial layer keydown)
                    for stolen_event in self.stolen_events.drain(..).skip(1) {
                        stolen_event.update_keycode(|keycode| match keycode {
                            AnsiKeyCode::S => KeyCode::LEFT_ARROW,
                            AnsiKeyCode::D => KeyCode::DOWN_ARROW,
                            AnsiKeyCode::F => KeyCode::RIGHT_ARROW,
                            AnsiKeyCode::E => KeyCode::UP_ARROW,
                            _ => unreachable!(),
                        });
                        stream.post(&stolen_event);
                    }
                    event.update_keycode(|keycode| match keycode {
                        AnsiKeyCode::S => KeyCode::LEFT_ARROW,
                        AnsiKeyCode::D => KeyCode::DOWN_ARROW,
                        AnsiKeyCode::F => KeyCode::RIGHT_ARROW,
                        AnsiKeyCode::E => KeyCode::UP_ARROW,
                        _ => unreachable!(),
                    });
                    stream.pass_current_event()
                }
            } else {
                // not a layer key
                stream.pass_current_event()
            }
        } else {
            // no layer pressed
            stream.pass_current_event()
        }
    }
}
