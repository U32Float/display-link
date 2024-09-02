use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use std::any::Any;
use std::ffi::c_void;

use crate::DisplayID;

use super::display_link_raw::DisplayLink as DisplayLinkRaw;
use super::display_link_raw::{CVDisplayLink, CVTimeStamp};

// -----------------------------------------------------------------------------

const _APPLE_TIME_OFFSET: i64 = 978307200;

unsafe extern "C" fn render<F>(
    _: *mut CVDisplayLink,
    _in_now: *const CVTimeStamp,
    _in_out_timestamp: *const CVTimeStamp,
    _: i64,
    _: *mut i64,
    display_link_context: *mut c_void,
) -> i32
where
    F: FnMut(),
{
    let f = &mut *(display_link_context as *mut F);
    f();

    0
}

// -----------------------------------------------------------------------------

pub struct DisplayLink {
    link_raw: *mut DisplayLinkRaw,
    callback: Option<Box<dyn Any>>,
    is_paused: bool,
}

unsafe impl Send for DisplayLink {}
unsafe impl Sync for DisplayLink {}

impl DisplayLink {
    pub fn on_display(display: DisplayID) -> Result<Self> {
        let display_link_raw = unsafe {
            DisplayLinkRaw::on_display(display).ok_or(anyhow!(
                "Failed to create display link for display: {}",
                display
            ))?
        };

        Ok(Self {
            link_raw: Box::into_raw(Box::new(display_link_raw)),
            callback: None,
            is_paused: true,
        })
    }

    pub fn set_callback<F>(&mut self, callback: F)
    where
        F: 'static + FnMut() + Send,
    {
        let raw = Box::into_raw(Box::new(callback));
        unsafe {
            (*self.link_raw).set_output_callback(render::<F>, raw as *mut c_void);
            self.callback = Some(Box::from_raw(raw));
        }
    }

    pub fn is_paused(&self) -> bool {
        self.is_paused
    }

    pub fn pause(&mut self) -> Result<()> {
        if self.is_paused {
            bail!("Display link is already paused");
        } else {
            unsafe {
                (*self.link_raw).stop();
                self.is_paused = true;
                Ok(())
            }
        }
    }

    pub fn resume(&mut self) -> Result<()> {
        if !self.is_paused {
            bail!("Display link is already running");
        } else {
            unsafe {
                (*self.link_raw).start();
                self.is_paused = false;
                Ok(())
            }
        }
    }
}

impl Drop for DisplayLink {
    fn drop(&mut self) {
        unsafe {
            let _ = Box::from_raw(self.link_raw);
        }
    }
}
