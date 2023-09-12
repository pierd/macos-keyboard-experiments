mod cgevent_tap_event_stream;
pub mod event_stream;
pub mod event_tap;
pub mod keycodes;

use cgevent_tap_event_stream::CGEventStream;
use core_foundation::{
    base::TCFType,
    runloop::{kCFRunLoopCommonModes, CFRunLoopAddSource, CFRunLoopGetCurrent, CFRunLoopRun},
};
use core_graphics::event::{
    CGEvent, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType,
};
use event_stream::EventType;
use event_tap::CGEventTap;
pub use parking_lot::Mutex;

use crate::event_tap::CGEventTapActiveFilterDecision;

pub fn start_event_tap<P: event_stream::EventProcessor<CGEvent>>(processor: Mutex<P>) {
    let tap = CGEventTap::new(
        CGEventTapLocation::Session,
        CGEventTapPlacement::HeadInsertEventTap,
        CGEventTapOptions::Default,
        vec![CGEventType::KeyDown, CGEventType::KeyUp],
        |tap_proxy, event_type, event| {
            let timestamp = std::time::Instant::now();
            let event_type = match event_type {
                CGEventType::KeyDown => EventType::KeyDown,
                CGEventType::KeyUp => EventType::KeyUp,
                _ => {
                    eprintln!("Unknown event type: {:?}", event_type);
                    return CGEventTapActiveFilterDecision::Pass;
                }
            };

            processor.lock().process(
                timestamp,
                event_type,
                event,
                CGEventStream::new(tap_proxy, event),
            )
        },
    )
    .unwrap();
    let runloop_source = tap.mach_port.create_runloop_source(0).unwrap();
    unsafe {
        CFRunLoopAddSource(
            CFRunLoopGetCurrent(),
            runloop_source.as_concrete_TypeRef(),
            kCFRunLoopCommonModes,
        );
    };
    tap.enable();
    unsafe { CFRunLoopRun() };
}
