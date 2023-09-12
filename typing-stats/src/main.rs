use keyboard_events_processing::Mutex;

mod stats_processor;

fn main() {
    let stats = Mutex::new(stats_processor::StatsProcessor::default());
    keyboard_events_processing::start_event_tap(stats);
}
