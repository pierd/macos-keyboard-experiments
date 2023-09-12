use keyboard_events_processing::Mutex;

mod hardcoded_processor;

fn main() {
    let handler = Mutex::new(hardcoded_processor::HardcodedProcessor::default());
    keyboard_events_processing::start_event_tap(handler);
}
