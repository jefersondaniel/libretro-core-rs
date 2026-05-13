use std::ffi::{CStr, CString};
use std::marker::PhantomPinned;
use std::pin::Pin;

use enumflags2::{BitFlags, bitflags};

use crate::raw::{
    self, RETRO_SIMD_AES, RETRO_SIMD_ASIMD, RETRO_SIMD_AVX, RETRO_SIMD_AVX2, RETRO_SIMD_CMOV,
    RETRO_SIMD_MMX, RETRO_SIMD_MMXEXT, RETRO_SIMD_MOVBE, RETRO_SIMD_NEON, RETRO_SIMD_POPCNT,
    RETRO_SIMD_PS, RETRO_SIMD_SSE, RETRO_SIMD_SSE2, RETRO_SIMD_SSE3, RETRO_SIMD_SSE4,
    RETRO_SIMD_SSE42, RETRO_SIMD_SSSE3, RETRO_SIMD_VFPU, RETRO_SIMD_VFPV3, RETRO_SIMD_VFPV4,
    RETRO_SIMD_VMX, RETRO_SIMD_VMX128,
};

#[bitflags]
#[repr(u64)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CpuFeature {
    Sse = RETRO_SIMD_SSE,
    Sse2 = RETRO_SIMD_SSE2,
    Vmx = RETRO_SIMD_VMX,
    Vmx128 = RETRO_SIMD_VMX128,
    Avx = RETRO_SIMD_AVX,
    Neon = RETRO_SIMD_NEON,
    Sse3 = RETRO_SIMD_SSE3,
    Ssse3 = RETRO_SIMD_SSSE3,
    Mmx = RETRO_SIMD_MMX,
    MmxExt = RETRO_SIMD_MMXEXT,
    Sse4 = RETRO_SIMD_SSE4,
    Sse42 = RETRO_SIMD_SSE42,
    Avx2 = RETRO_SIMD_AVX2,
    Vfpu = RETRO_SIMD_VFPU,
    Ps = RETRO_SIMD_PS,
    Aes = RETRO_SIMD_AES,
    Vfpv3 = RETRO_SIMD_VFPV3,
    Vfpv4 = RETRO_SIMD_VFPV4,
    Popcnt = RETRO_SIMD_POPCNT,
    Movbe = RETRO_SIMD_MOVBE,
    Cmov = RETRO_SIMD_CMOV,
    Asimd = RETRO_SIMD_ASIMD,
}

pub type CpuFeatures = BitFlags<CpuFeature>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PerfTick(u64);

impl PerfTick {
    pub const fn from_ticks(ticks: u64) -> Self {
        Self(ticks)
    }

    pub const fn as_ticks(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PerfTimeMicros(i64);

impl PerfTimeMicros {
    pub const fn from_micros(micros: i64) -> Self {
        Self(micros)
    }

    pub const fn as_micros(self) -> i64 {
        self.0
    }
}

#[derive(Debug)]
pub struct PerfCounter {
    ident: CString,
    raw: raw::retro_perf_counter,
    _pin: PhantomPinned,
}

impl PerfCounter {
    pub fn new(ident: impl AsRef<str>) -> Pin<Box<Self>> {
        let ident = crate::sanitize_cstring(ident);
        let raw = raw::retro_perf_counter {
            ident: ident.as_ptr(),
            ..raw::retro_perf_counter::default()
        };

        Box::pin(Self {
            ident,
            raw,
            _pin: PhantomPinned,
        })
    }

    pub fn ident(&self) -> &CStr {
        self.ident.as_c_str()
    }

    pub fn is_registered(&self) -> bool {
        self.raw.registered
    }

    pub fn last_start(&self) -> PerfTick {
        PerfTick::from_ticks(self.raw.start)
    }

    pub fn total(&self) -> PerfTick {
        PerfTick::from_ticks(self.raw.total)
    }

    pub fn call_count(&self) -> u64 {
        self.raw.call_cnt
    }

    fn as_raw_mut(self: Pin<&mut Self>) -> *mut raw::retro_perf_counter {
        // SAFETY: This method never moves the pinned counter; it only exposes
        // its stable raw storage to the frontend callback for in-place updates.
        unsafe { &mut self.get_unchecked_mut().raw }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PerfInterface {
    raw: raw::retro_perf_callback,
}

impl PerfInterface {
    pub(crate) const fn from_raw(raw: raw::retro_perf_callback) -> Self {
        Self { raw }
    }

    pub fn time_micros(&self) -> Option<PerfTimeMicros> {
        self.raw
            .get_time_usec
            .map(|get_time_usec| PerfTimeMicros::from_micros(unsafe { get_time_usec() }))
    }

    pub fn tick_counter(&self) -> Option<PerfTick> {
        self.raw
            .get_perf_counter
            .map(|get_perf_counter| PerfTick::from_ticks(unsafe { get_perf_counter() }))
    }

    pub fn cpu_features(&self) -> Option<CpuFeatures> {
        self.raw
            .get_cpu_features
            .map(|get_cpu_features| CpuFeatures::from_bits_truncate(unsafe { get_cpu_features() }))
    }

    pub fn log(&self) -> bool {
        if let Some(perf_log) = self.raw.perf_log {
            unsafe { perf_log() };
            true
        } else {
            false
        }
    }

    pub fn register_counter(&self, mut counter: Pin<&mut PerfCounter>) -> bool {
        let Some(perf_register) = self.raw.perf_register else {
            return false;
        };

        unsafe { perf_register(counter.as_mut().as_raw_mut()) };
        counter.as_ref().get_ref().is_registered()
    }

    pub fn start_counter(&self, mut counter: Pin<&mut PerfCounter>) -> bool {
        let Some(perf_start) = self.raw.perf_start else {
            return false;
        };
        if !counter.as_ref().get_ref().is_registered() {
            return false;
        }

        unsafe { perf_start(counter.as_mut().as_raw_mut()) };
        true
    }

    pub fn stop_counter(&self, mut counter: Pin<&mut PerfCounter>) -> bool {
        let Some(perf_stop) = self.raw.perf_stop else {
            return false;
        };
        if !counter.as_ref().get_ref().is_registered() {
            return false;
        }

        unsafe { perf_stop(counter.as_mut().as_raw_mut()) };
        true
    }
}

#[cfg(test)]
mod tests {
    use super::{CpuFeature, CpuFeatures, PerfCounter, PerfTick, PerfTimeMicros};

    #[test]
    fn cpu_features_encode_libretro_simd_bits() {
        let features = CpuFeatures::from(CpuFeature::Sse2) | CpuFeature::Avx2 | CpuFeature::Neon;

        assert!(features.contains(CpuFeature::Sse2));
        assert!(features.contains(CpuFeature::Avx2));
        assert!(features.contains(CpuFeature::Neon));
        assert_eq!(
            features.bits(),
            crate::raw::RETRO_SIMD_SSE2 | crate::raw::RETRO_SIMD_AVX2 | crate::raw::RETRO_SIMD_NEON
        );
    }

    #[test]
    fn perf_time_and_tick_newtypes_preserve_raw_values() {
        assert_eq!(PerfTimeMicros::from_micros(-1).as_micros(), -1);
        assert_eq!(PerfTick::from_ticks(42).as_ticks(), 42);
    }

    #[test]
    fn perf_counter_owns_sanitized_identifier_storage() {
        let counter = PerfCounter::new("hot\0path");

        assert_eq!(
            counter
                .ident()
                .to_str()
                .expect("identifier should be utf-8"),
            "hotpath"
        );
        assert!(!counter.is_registered());
        assert_eq!(counter.call_count(), 0);
        assert_eq!(counter.total(), PerfTick::from_ticks(0));
    }
}
