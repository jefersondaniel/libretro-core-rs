use std::marker::PhantomData;
use std::ptr::NonNull;

use crate::raw;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct MicrophoneRateHz(u32);

impl MicrophoneRateHz {
    pub const fn new(rate: u32) -> Self {
        Self(rate)
    }

    pub const fn frontend_default() -> Self {
        Self(0)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct MicrophoneParams {
    pub rate: MicrophoneRateHz,
}

impl MicrophoneParams {
    pub const fn new(rate: MicrophoneRateHz) -> Self {
        Self { rate }
    }

    pub const fn frontend_default() -> Self {
        Self {
            rate: MicrophoneRateHz::frontend_default(),
        }
    }

    pub(crate) const fn into_raw(self) -> raw::retro_microphone_params {
        raw::retro_microphone_params {
            rate: self.rate.get(),
        }
    }

    pub(crate) const fn from_raw(raw: raw::retro_microphone_params) -> Self {
        Self {
            rate: MicrophoneRateHz::new(raw.rate),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MicrophoneInterface {
    raw: raw::retro_microphone_interface,
}

impl MicrophoneInterface {
    pub(crate) const fn from_raw(raw: raw::retro_microphone_interface) -> Self {
        Self { raw }
    }

    pub const fn version(self) -> u32 {
        self.raw.interface_version
    }

    pub const fn is_available(self) -> bool {
        self.raw.interface_version >= raw::RETRO_MICROPHONE_INTERFACE_VERSION
            && self.raw.open_mic.is_some()
            && self.raw.close_mic.is_some()
            && self.raw.get_params.is_some()
            && self.raw.set_mic_state.is_some()
            && self.raw.get_mic_state.is_some()
            && self.raw.read_mic.is_some()
    }

    pub fn open_default(self) -> Option<Microphone> {
        self.open_raw(None)
    }

    pub fn open(self, params: MicrophoneParams) -> Option<Microphone> {
        self.open_raw(Some(params.into_raw()))
    }

    fn open_raw(self, params: Option<raw::retro_microphone_params>) -> Option<Microphone> {
        let open_mic = self.raw.open_mic?;
        let handle = match params {
            Some(params) => unsafe { open_mic(&params) },
            None => unsafe { open_mic(std::ptr::null()) },
        };
        Some(Microphone {
            handle: NonNull::new(handle)?,
            interface: self,
            _not_send_sync: PhantomData,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum MicrophoneReadError {
    Unavailable,
    ReturnedTooManySamples { returned: usize, capacity: usize },
}

#[derive(Debug)]
pub struct Microphone {
    handle: NonNull<raw::retro_microphone>,
    interface: MicrophoneInterface,
    _not_send_sync: PhantomData<*mut ()>,
}

impl Microphone {
    pub fn params(&self) -> Option<MicrophoneParams> {
        let get_params = self.interface.raw.get_params?;
        let mut params = raw::retro_microphone_params::default();
        if unsafe { get_params(self.handle.as_ptr().cast_const(), &mut params) } {
            Some(MicrophoneParams::from_raw(params))
        } else {
            None
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) -> bool {
        self.interface
            .raw
            .set_mic_state
            .map(|set_mic_state| unsafe { set_mic_state(self.handle.as_ptr(), enabled) })
            .unwrap_or(false)
    }

    pub fn enabled(&self) -> bool {
        self.interface
            .raw
            .get_mic_state
            .map(|get_mic_state| unsafe { get_mic_state(self.handle.as_ptr().cast_const()) })
            .unwrap_or(false)
    }

    pub fn read_samples(&mut self, samples: &mut [i16]) -> Result<usize, MicrophoneReadError> {
        let Some(read_mic) = self.interface.raw.read_mic else {
            return Err(MicrophoneReadError::Unavailable);
        };
        let read = unsafe { read_mic(self.handle.as_ptr(), samples.as_mut_ptr(), samples.len()) };
        let Ok(read) = usize::try_from(read) else {
            return Err(MicrophoneReadError::Unavailable);
        };
        if read > samples.len() {
            return Err(MicrophoneReadError::ReturnedTooManySamples {
                returned: read,
                capacity: samples.len(),
            });
        }
        Ok(read)
    }
}

impl Drop for Microphone {
    fn drop(&mut self) {
        if let Some(close_mic) = self.interface.raw.close_mic {
            unsafe { close_mic(self.handle.as_ptr()) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{MicrophoneInterface, MicrophoneParams, MicrophoneRateHz, MicrophoneReadError};

    #[test]
    fn microphone_params_preserve_frontend_default_rate() {
        assert_eq!(MicrophoneRateHz::new(16_000).get(), 16_000);
        assert_eq!(MicrophoneRateHz::frontend_default().get(), 0);
        assert_eq!(MicrophoneParams::frontend_default().rate.get(), 0);
    }

    #[test]
    fn empty_microphone_interface_reports_unavailable() {
        let interface = MicrophoneInterface::default();

        assert_eq!(interface.version(), 0);
        assert!(!interface.is_available());
        assert!(interface.open_default().is_none());
        assert!(
            interface
                .open(MicrophoneParams::frontend_default())
                .is_none()
        );
    }

    #[test]
    fn microphone_read_error_preserves_oversized_frontend_result() {
        assert_eq!(
            MicrophoneReadError::ReturnedTooManySamples {
                returned: 4,
                capacity: 2,
            },
            MicrophoneReadError::ReturnedTooManySamples {
                returned: 4,
                capacity: 2,
            }
        );
    }
}
