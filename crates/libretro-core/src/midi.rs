use crate::raw;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct MidiDeltaMicros(u32);

impl MidiDeltaMicros {
    pub const fn new(micros: u32) -> Self {
        Self(micros)
    }

    pub const fn as_micros(self) -> u32 {
        self.0
    }
}

impl From<u32> for MidiDeltaMicros {
    fn from(micros: u32) -> Self {
        Self::new(micros)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MidiInterface {
    raw: raw::retro_midi_interface,
}

impl MidiInterface {
    pub(crate) const fn from_raw(raw: raw::retro_midi_interface) -> Self {
        Self { raw }
    }

    pub const fn is_available(self) -> bool {
        self.raw.input_enabled.is_some()
            && self.raw.output_enabled.is_some()
            && self.raw.read.is_some()
            && self.raw.write.is_some()
            && self.raw.flush.is_some()
    }

    pub fn input_enabled(self) -> bool {
        self.raw
            .input_enabled
            .map(|input_enabled| unsafe { input_enabled() })
            .unwrap_or(false)
    }

    pub fn output_enabled(self) -> bool {
        self.raw
            .output_enabled
            .map(|output_enabled| unsafe { output_enabled() })
            .unwrap_or(false)
    }

    pub fn read_byte(self) -> Option<u8> {
        let read = self.raw.read?;
        let mut byte = 0u8;
        if unsafe { read(&mut byte as *mut u8) } {
            Some(byte)
        } else {
            None
        }
    }

    pub fn write_byte(self, byte: u8, delta_time: impl Into<MidiDeltaMicros>) -> bool {
        self.raw
            .write
            .map(|write| unsafe { write(byte, delta_time.into().as_micros()) })
            .unwrap_or(false)
    }

    pub fn flush(self) -> bool {
        self.raw
            .flush
            .map(|flush| unsafe { flush() })
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::{MidiDeltaMicros, MidiInterface};

    #[test]
    fn midi_delta_micros_preserves_raw_value() {
        assert_eq!(MidiDeltaMicros::new(240).as_micros(), 240);
    }

    #[test]
    fn empty_midi_interface_reports_unavailable() {
        let midi = MidiInterface::default();

        assert!(!midi.is_available());
        assert!(!midi.input_enabled());
        assert!(!midi.output_enabled());
        assert_eq!(midi.read_byte(), None);
        assert!(!midi.write_byte(0x90, MidiDeltaMicros::new(0)));
        assert!(!midi.flush());
    }
}
