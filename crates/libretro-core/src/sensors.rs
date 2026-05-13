//! Sensor and location service-interface wrappers.
//!
//! Sensor polling is a typed frontend service. Location lifecycle callbacks are
//! event-shaped and should be registered through `CoreEventConfig`.

use crate::{InputPort, raw};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sensor {
    Accelerometer,
    Gyroscope,
    Illuminance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SensorAction {
    Enable(Sensor),
    Disable(Sensor),
}

impl SensorAction {
    pub(crate) const fn as_raw(self) -> raw::retro_sensor_action {
        match self {
            Self::Enable(Sensor::Accelerometer) => raw::retro_sensor_action::AccelerometerEnable,
            Self::Disable(Sensor::Accelerometer) => raw::retro_sensor_action::AccelerometerDisable,
            Self::Enable(Sensor::Gyroscope) => raw::retro_sensor_action::GyroscopeEnable,
            Self::Disable(Sensor::Gyroscope) => raw::retro_sensor_action::GyroscopeDisable,
            Self::Enable(Sensor::Illuminance) => raw::retro_sensor_action::IlluminanceEnable,
            Self::Disable(Sensor::Illuminance) => raw::retro_sensor_action::IlluminanceDisable,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct SensorRateHz(u32);

impl SensorRateHz {
    pub const fn new(rate: u32) -> Self {
        Self(rate)
    }

    pub const fn as_raw(self) -> u32 {
        self.0
    }
}

impl From<u32> for SensorRateHz {
    fn from(rate: u32) -> Self {
        Self::new(rate)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct LocationIntervalMillis(u32);

impl LocationIntervalMillis {
    pub const fn new(milliseconds: u32) -> Self {
        Self(milliseconds)
    }

    pub const fn as_millis(self) -> u32 {
        self.0
    }
}

impl From<u32> for LocationIntervalMillis {
    fn from(milliseconds: u32) -> Self {
        Self::new(milliseconds)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct LocationIntervalMeters(u32);

impl LocationIntervalMeters {
    pub const fn new(meters: u32) -> Self {
        Self(meters)
    }

    pub const fn as_meters(self) -> u32 {
        self.0
    }
}

impl From<u32> for LocationIntervalMeters {
    fn from(meters: u32) -> Self {
        Self::new(meters)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LocationPosition {
    pub latitude_degrees: f64,
    pub longitude_degrees: f64,
    pub horizontal_accuracy: f64,
    pub vertical_accuracy: f64,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct LocationInterface {
    raw: raw::retro_location_callback,
}

impl LocationInterface {
    pub(crate) const fn from_raw(raw: raw::retro_location_callback) -> Self {
        Self { raw }
    }

    pub const fn is_available(self) -> bool {
        self.raw.start.is_some()
            && self.raw.stop.is_some()
            && self.raw.get_position.is_some()
            && self.raw.set_interval.is_some()
    }

    pub fn start(self) -> bool {
        self.raw.start.is_some_and(|start| unsafe { start() })
    }

    pub fn stop(self) -> bool {
        let Some(stop) = self.raw.stop else {
            return false;
        };
        unsafe { stop() };
        true
    }

    pub fn set_interval(
        self,
        interval: impl Into<LocationIntervalMillis>,
        distance: impl Into<LocationIntervalMeters>,
    ) -> bool {
        let Some(set_interval) = self.raw.set_interval else {
            return false;
        };
        unsafe { set_interval(interval.into().as_millis(), distance.into().as_meters()) };
        true
    }

    pub fn position(self) -> Option<LocationPosition> {
        let get_position = self.raw.get_position?;
        let mut latitude_degrees = 0.0;
        let mut longitude_degrees = 0.0;
        let mut horizontal_accuracy = 0.0;
        let mut vertical_accuracy = 0.0;
        if unsafe {
            get_position(
                &mut latitude_degrees,
                &mut longitude_degrees,
                &mut horizontal_accuracy,
                &mut vertical_accuracy,
            )
        } {
            Some(LocationPosition {
                latitude_degrees,
                longitude_degrees,
                horizontal_accuracy,
                vertical_accuracy,
            })
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SensorInput {
    AccelerometerX,
    AccelerometerY,
    AccelerometerZ,
    GyroscopeX,
    GyroscopeY,
    GyroscopeZ,
    Illuminance,
}

impl SensorInput {
    pub const fn as_raw(self) -> u32 {
        match self {
            Self::AccelerometerX => raw::RETRO_SENSOR_ACCELEROMETER_X,
            Self::AccelerometerY => raw::RETRO_SENSOR_ACCELEROMETER_Y,
            Self::AccelerometerZ => raw::RETRO_SENSOR_ACCELEROMETER_Z,
            Self::GyroscopeX => raw::RETRO_SENSOR_GYROSCOPE_X,
            Self::GyroscopeY => raw::RETRO_SENSOR_GYROSCOPE_Y,
            Self::GyroscopeZ => raw::RETRO_SENSOR_GYROSCOPE_Z,
            Self::Illuminance => raw::RETRO_SENSOR_ILLUMINANCE,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SensorInterface {
    raw: raw::retro_sensor_interface,
}

impl SensorInterface {
    pub(crate) const fn from_raw(raw: raw::retro_sensor_interface) -> Self {
        Self { raw }
    }

    pub const fn is_available(self) -> bool {
        self.raw.set_sensor_state.is_some() && self.raw.get_sensor_input.is_some()
    }

    pub fn set_state(
        self,
        port: impl Into<InputPort>,
        action: SensorAction,
        rate: impl Into<SensorRateHz>,
    ) -> bool {
        let Some(set_sensor_state) = self.raw.set_sensor_state else {
            return false;
        };

        unsafe { set_sensor_state(port.into().as_raw(), action.as_raw(), rate.into().as_raw()) }
    }

    pub fn enable(
        self,
        port: impl Into<InputPort>,
        sensor: Sensor,
        rate: impl Into<SensorRateHz>,
    ) -> bool {
        self.set_state(port, SensorAction::Enable(sensor), rate)
    }

    pub fn disable(self, port: impl Into<InputPort>, sensor: Sensor) -> bool {
        self.set_state(port, SensorAction::Disable(sensor), SensorRateHz::new(0))
    }

    pub fn input(self, port: impl Into<InputPort>, input: SensorInput) -> Option<f32> {
        self.raw.get_sensor_input.map(|get_sensor_input| unsafe {
            get_sensor_input(port.into().as_raw(), input.as_raw())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LocationInterface, LocationIntervalMeters, LocationIntervalMillis, LocationPosition,
        Sensor, SensorAction, SensorInput, SensorInterface, SensorRateHz,
    };

    #[test]
    fn sensor_actions_encode_libretro_values() {
        assert_eq!(
            SensorAction::Enable(Sensor::Accelerometer).as_raw(),
            crate::raw::retro_sensor_action::AccelerometerEnable
        );
        assert_eq!(
            SensorAction::Disable(Sensor::Illuminance).as_raw(),
            crate::raw::retro_sensor_action::IlluminanceDisable
        );
    }

    #[test]
    fn sensor_inputs_encode_libretro_ids() {
        assert_eq!(
            SensorInput::AccelerometerX.as_raw(),
            crate::raw::RETRO_SENSOR_ACCELEROMETER_X
        );
        assert_eq!(
            SensorInput::Illuminance.as_raw(),
            crate::raw::RETRO_SENSOR_ILLUMINANCE
        );
    }

    #[test]
    fn empty_sensor_interface_reports_unavailable() {
        let sensors = SensorInterface::default();

        assert!(!sensors.is_available());
        assert!(!sensors.enable(0, Sensor::Accelerometer, SensorRateHz::new(60)));
        assert_eq!(sensors.input(0, SensorInput::AccelerometerX), None);
    }

    #[test]
    fn empty_location_interface_reports_unavailable() {
        let location = LocationInterface::default();

        assert!(!location.is_available());
        assert!(!location.start());
        assert!(!location.stop());
        assert!(!location.set_interval(LocationIntervalMillis::new(1000), 10));
        assert_eq!(location.position(), None);
    }

    #[test]
    fn location_interval_newtypes_preserve_units() {
        assert_eq!(LocationIntervalMillis::new(500).as_millis(), 500);
        assert_eq!(LocationIntervalMeters::new(20).as_meters(), 20);
        assert_eq!(
            LocationPosition {
                latitude_degrees: 1.0,
                longitude_degrees: 2.0,
                horizontal_accuracy: 3.0,
                vertical_accuracy: 4.0,
            }
            .horizontal_accuracy,
            3.0
        );
    }
}
