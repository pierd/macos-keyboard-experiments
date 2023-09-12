use foreign_types::ForeignType;
use libc::c_void;
use std::{mem::ManuallyDrop, ptr};

use core_foundation::{
    base::TCFType,
    mach_port::{CFMachPort, CFMachPortRef},
};
use core_graphics::{
    event::{
        CGEvent, CGEventMask, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
        CGEventTapProxy, CGEventType,
    },
    sys,
};

type CGEventTapCallBackInternal = unsafe extern "C" fn(
    proxy: CGEventTapProxy,
    etype: CGEventType,
    event: sys::CGEventRef,
    user_info: *const c_void,
) -> sys::CGEventRef;

#[derive(Default)]
pub enum CGEventTapActiveFilterDecision {
    #[default]
    Pass,
    Override(CGEvent),
    Drop,
}

// https://developer.apple.com/documentation/coregraphics/cgeventtapcallback
pub type CGEventTapCallBackFn<'tap_life> = Box<
    dyn Fn(CGEventTapProxy, CGEventType, &CGEvent) -> CGEventTapActiveFilterDecision + 'tap_life,
>;

unsafe extern "C" fn cg_event_tap_callback_internal(
    _proxy: CGEventTapProxy,
    _etype: CGEventType,
    _event: sys::CGEventRef,
    _user_info: *const c_void,
) -> sys::CGEventRef {
    let callback = _user_info as *mut CGEventTapCallBackFn;
    let event = CGEvent::from_ptr(_event);
    let decision = (*callback)(_proxy, _etype, &event);
    let event = ManuallyDrop::new(event);
    match decision {
        CGEventTapActiveFilterDecision::Pass => event.as_ptr(),
        CGEventTapActiveFilterDecision::Override(new_event) => {
            ManuallyDrop::new(new_event).as_ptr()
        }
        CGEventTapActiveFilterDecision::Drop => ptr::null_mut(),
    }
}

macro_rules! CGEventMaskBit {
    ($eventType:expr) => {
        1 << $eventType as CGEventMask
    };
}

pub struct CGEventTap<'tap_life> {
    pub mach_port: CFMachPort,
    pub callback_ref: CGEventTapCallBackFn<'tap_life>,
}

impl<'tap_life> CGEventTap<'tap_life> {
    #[allow(clippy::result_unit_err)]
    pub fn new<
        F: Fn(CGEventTapProxy, CGEventType, &CGEvent) -> CGEventTapActiveFilterDecision + 'tap_life,
    >(
        tap: CGEventTapLocation,
        place: CGEventTapPlacement,
        options: CGEventTapOptions,
        events_of_interest: std::vec::Vec<CGEventType>,
        callback: F,
    ) -> Result<CGEventTap<'tap_life>, ()> {
        let event_mask: CGEventMask = events_of_interest
            .iter()
            .fold(CGEventType::Null as CGEventMask, |mask, &etype| -> u64 {
                mask | CGEventMaskBit!(etype)
            });
        let cb = Box::new(Box::new(callback) as CGEventTapCallBackFn);
        let cbr = Box::into_raw(cb);
        unsafe {
            let event_tap_ref = CGEventTapCreate(
                tap,
                place,
                options,
                event_mask,
                cg_event_tap_callback_internal,
                cbr as *const c_void,
            );

            if !event_tap_ref.is_null() {
                Ok(Self {
                    mach_port: (CFMachPort::wrap_under_create_rule(event_tap_ref)),
                    callback_ref: Box::from_raw(cbr),
                })
            } else {
                let _ = Box::from_raw(cbr);
                Err(())
            }
        }
    }

    pub fn enable(&self) {
        unsafe { CGEventTapEnable(self.mach_port.as_concrete_TypeRef(), true) }
    }
}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    // ::sys::CGEventTapRef is actually an CFMachPortRef
    fn CGEventTapCreate(
        tap: CGEventTapLocation,
        place: CGEventTapPlacement,
        options: CGEventTapOptions,
        eventsOfInterest: CGEventMask,
        callback: CGEventTapCallBackInternal,
        userInfo: *const c_void,
    ) -> CFMachPortRef;

    fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
}
