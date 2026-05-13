use enumflags2::{BitFlags, bitflags};

use crate::raw::{
    RETRO_DEVICE_ANALOG, RETRO_DEVICE_ID_ANALOG_X, RETRO_DEVICE_ID_ANALOG_Y,
    RETRO_DEVICE_ID_JOYPAD_A, RETRO_DEVICE_ID_JOYPAD_B, RETRO_DEVICE_ID_JOYPAD_DOWN,
    RETRO_DEVICE_ID_JOYPAD_L, RETRO_DEVICE_ID_JOYPAD_L2, RETRO_DEVICE_ID_JOYPAD_L3,
    RETRO_DEVICE_ID_JOYPAD_LEFT, RETRO_DEVICE_ID_JOYPAD_MASK, RETRO_DEVICE_ID_JOYPAD_R,
    RETRO_DEVICE_ID_JOYPAD_R2, RETRO_DEVICE_ID_JOYPAD_R3, RETRO_DEVICE_ID_JOYPAD_RIGHT,
    RETRO_DEVICE_ID_JOYPAD_SELECT, RETRO_DEVICE_ID_JOYPAD_START, RETRO_DEVICE_ID_JOYPAD_UP,
    RETRO_DEVICE_ID_JOYPAD_X, RETRO_DEVICE_ID_JOYPAD_Y, RETRO_DEVICE_ID_LIGHTGUN_AUX_A,
    RETRO_DEVICE_ID_LIGHTGUN_AUX_B, RETRO_DEVICE_ID_LIGHTGUN_AUX_C,
    RETRO_DEVICE_ID_LIGHTGUN_DPAD_DOWN, RETRO_DEVICE_ID_LIGHTGUN_DPAD_LEFT,
    RETRO_DEVICE_ID_LIGHTGUN_DPAD_RIGHT, RETRO_DEVICE_ID_LIGHTGUN_DPAD_UP,
    RETRO_DEVICE_ID_LIGHTGUN_IS_OFFSCREEN, RETRO_DEVICE_ID_LIGHTGUN_RELOAD,
    RETRO_DEVICE_ID_LIGHTGUN_SCREEN_X, RETRO_DEVICE_ID_LIGHTGUN_SCREEN_Y,
    RETRO_DEVICE_ID_LIGHTGUN_SELECT, RETRO_DEVICE_ID_LIGHTGUN_START,
    RETRO_DEVICE_ID_LIGHTGUN_TRIGGER, RETRO_DEVICE_ID_LIGHTGUN_X, RETRO_DEVICE_ID_LIGHTGUN_Y,
    RETRO_DEVICE_ID_MOUSE_BUTTON_4, RETRO_DEVICE_ID_MOUSE_BUTTON_5,
    RETRO_DEVICE_ID_MOUSE_HORIZ_WHEELDOWN, RETRO_DEVICE_ID_MOUSE_HORIZ_WHEELUP,
    RETRO_DEVICE_ID_MOUSE_LEFT, RETRO_DEVICE_ID_MOUSE_MIDDLE, RETRO_DEVICE_ID_MOUSE_RIGHT,
    RETRO_DEVICE_ID_MOUSE_WHEELDOWN, RETRO_DEVICE_ID_MOUSE_WHEELUP, RETRO_DEVICE_ID_MOUSE_X,
    RETRO_DEVICE_ID_MOUSE_Y, RETRO_DEVICE_ID_POINTER_COUNT, RETRO_DEVICE_ID_POINTER_IS_OFFSCREEN,
    RETRO_DEVICE_ID_POINTER_PRESSED, RETRO_DEVICE_ID_POINTER_X, RETRO_DEVICE_ID_POINTER_Y,
    RETRO_DEVICE_INDEX_ANALOG_BUTTON, RETRO_DEVICE_INDEX_ANALOG_LEFT,
    RETRO_DEVICE_INDEX_ANALOG_RIGHT, RETRO_DEVICE_JOYPAD, RETRO_DEVICE_KEYBOARD,
    RETRO_DEVICE_LIGHTGUN, RETRO_DEVICE_MASK, RETRO_DEVICE_MOUSE, RETRO_DEVICE_NONE,
    RETRO_DEVICE_POINTER, RETRO_DEVICE_TYPE_SHIFT,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct LedIndex(i32);

impl LedIndex {
    pub const fn new(index: i32) -> Self {
        Self(index)
    }

    pub const fn as_raw(self) -> i32 {
        self.0
    }
}

impl From<i32> for LedIndex {
    fn from(index: i32) -> Self {
        Self::new(index)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum LedState {
    #[default]
    Off,
    On,
}

impl LedState {
    pub const fn as_raw(self) -> i32 {
        match self {
            Self::Off => 0,
            Self::On => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct LedInterface {
    raw: crate::raw::retro_led_interface,
}

impl LedInterface {
    pub(crate) const fn from_raw(raw: crate::raw::retro_led_interface) -> Self {
        Self { raw }
    }

    pub const fn is_available(self) -> bool {
        self.raw.set_led_state.is_some()
    }

    pub fn set_state(self, led: impl Into<LedIndex>, state: LedState) -> bool {
        let Some(callback) = self.raw.set_led_state else {
            return false;
        };

        // SAFETY: The callback pointer is provided by the frontend through the
        // libretro LED interface. Arguments are plain value types.
        unsafe { callback(led.into().as_raw(), state.as_raw()) };
        true
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RumbleEffect {
    Strong,
    Weak,
}

impl RumbleEffect {
    pub(crate) const fn as_raw(self) -> crate::raw::retro_rumble_effect {
        match self {
            Self::Strong => crate::raw::retro_rumble_effect::Strong,
            Self::Weak => crate::raw::retro_rumble_effect::Weak,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct RumbleStrength(u16);

impl RumbleStrength {
    pub const fn new(strength: u16) -> Self {
        Self(strength)
    }

    pub const fn off() -> Self {
        Self(0)
    }

    pub const fn max() -> Self {
        Self(u16::MAX)
    }

    pub const fn as_raw(self) -> u16 {
        self.0
    }
}

impl From<u16> for RumbleStrength {
    fn from(strength: u16) -> Self {
        Self::new(strength)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct RumbleInterface {
    raw: crate::raw::retro_rumble_interface,
}

impl RumbleInterface {
    pub(crate) const fn from_raw(raw: crate::raw::retro_rumble_interface) -> Self {
        Self { raw }
    }

    pub const fn is_available(self) -> bool {
        self.raw.set_rumble_state.is_some()
    }

    pub fn set_state(
        self,
        port: impl Into<InputPort>,
        effect: RumbleEffect,
        strength: impl Into<RumbleStrength>,
    ) -> bool {
        let Some(callback) = self.raw.set_rumble_state else {
            return false;
        };

        // SAFETY: The callback pointer is provided by the frontend through the
        // libretro rumble interface. Arguments are plain value types.
        unsafe {
            callback(
                port.into().as_raw(),
                effect.as_raw(),
                strength.into().as_raw(),
            )
        }
    }
}

#[bitflags]
#[repr(u64)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InputDeviceCapability {
    Joypad = 1u64 << RETRO_DEVICE_JOYPAD,
    Mouse = 1u64 << RETRO_DEVICE_MOUSE,
    Keyboard = 1u64 << RETRO_DEVICE_KEYBOARD,
    Lightgun = 1u64 << RETRO_DEVICE_LIGHTGUN,
    Analog = 1u64 << RETRO_DEVICE_ANALOG,
    Pointer = 1u64 << RETRO_DEVICE_POINTER,
}

pub type InputDeviceCapabilities = BitFlags<InputDeviceCapability>;

impl InputDeviceCapability {
    pub const fn device(self) -> ControllerDevice {
        match self {
            Self::Joypad => ControllerDevice::Joypad,
            Self::Mouse => ControllerDevice::Mouse,
            Self::Keyboard => ControllerDevice::Keyboard,
            Self::Lightgun => ControllerDevice::Lightgun,
            Self::Analog => ControllerDevice::Analog,
            Self::Pointer => ControllerDevice::Pointer,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum KeyboardKey {
    #[default]
    Unknown,
    Backspace,
    Tab,
    Clear,
    Return,
    Pause,
    Escape,
    Space,
    Exclaim,
    QuoteDbl,
    Hash,
    Dollar,
    Ampersand,
    Quote,
    LeftParen,
    RightParen,
    Asterisk,
    Plus,
    Comma,
    Minus,
    Period,
    Slash,
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Colon,
    Semicolon,
    Less,
    Equals,
    Greater,
    Question,
    At,
    LeftBracket,
    Backslash,
    RightBracket,
    Caret,
    Underscore,
    Backquote,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    LeftBrace,
    Bar,
    RightBrace,
    Tilde,
    Delete,
    Kp0,
    Kp1,
    Kp2,
    Kp3,
    Kp4,
    Kp5,
    Kp6,
    Kp7,
    Kp8,
    Kp9,
    KpPeriod,
    KpDivide,
    KpMultiply,
    KpMinus,
    KpPlus,
    KpEnter,
    KpEquals,
    Up,
    Down,
    Right,
    Left,
    Insert,
    Home,
    End,
    PageUp,
    PageDown,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    NumLock,
    CapsLock,
    ScrollLock,
    RightShift,
    LeftShift,
    RightCtrl,
    LeftCtrl,
    RightAlt,
    LeftAlt,
    RightMeta,
    LeftMeta,
    LeftSuper,
    RightSuper,
    Mode,
    Compose,
    Help,
    Print,
    SysReq,
    Break,
    Menu,
    Power,
    Euro,
    Undo,
    Oem102,
    BrowserBack,
    BrowserForward,
    BrowserRefresh,
    BrowserStop,
    BrowserSearch,
    BrowserFavorites,
    BrowserHome,
    VolumeMute,
    VolumeDown,
    VolumeUp,
    MediaNext,
    MediaPrev,
    MediaStop,
    MediaPlayPause,
    LaunchMail,
    LaunchMedia,
    LaunchApp1,
    LaunchApp2,
    Last,
    UnknownKeycode(u32),
}

impl KeyboardKey {
    pub const fn from_raw(keycode: u32) -> Self {
        match keycode {
            0 => Self::Unknown,
            8 => Self::Backspace,
            9 => Self::Tab,
            12 => Self::Clear,
            13 => Self::Return,
            19 => Self::Pause,
            27 => Self::Escape,
            32 => Self::Space,
            33 => Self::Exclaim,
            34 => Self::QuoteDbl,
            35 => Self::Hash,
            36 => Self::Dollar,
            38 => Self::Ampersand,
            39 => Self::Quote,
            40 => Self::LeftParen,
            41 => Self::RightParen,
            42 => Self::Asterisk,
            43 => Self::Plus,
            44 => Self::Comma,
            45 => Self::Minus,
            46 => Self::Period,
            47 => Self::Slash,
            48 => Self::Num0,
            49 => Self::Num1,
            50 => Self::Num2,
            51 => Self::Num3,
            52 => Self::Num4,
            53 => Self::Num5,
            54 => Self::Num6,
            55 => Self::Num7,
            56 => Self::Num8,
            57 => Self::Num9,
            58 => Self::Colon,
            59 => Self::Semicolon,
            60 => Self::Less,
            61 => Self::Equals,
            62 => Self::Greater,
            63 => Self::Question,
            64 => Self::At,
            91 => Self::LeftBracket,
            92 => Self::Backslash,
            93 => Self::RightBracket,
            94 => Self::Caret,
            95 => Self::Underscore,
            96 => Self::Backquote,
            97 => Self::A,
            98 => Self::B,
            99 => Self::C,
            100 => Self::D,
            101 => Self::E,
            102 => Self::F,
            103 => Self::G,
            104 => Self::H,
            105 => Self::I,
            106 => Self::J,
            107 => Self::K,
            108 => Self::L,
            109 => Self::M,
            110 => Self::N,
            111 => Self::O,
            112 => Self::P,
            113 => Self::Q,
            114 => Self::R,
            115 => Self::S,
            116 => Self::T,
            117 => Self::U,
            118 => Self::V,
            119 => Self::W,
            120 => Self::X,
            121 => Self::Y,
            122 => Self::Z,
            123 => Self::LeftBrace,
            124 => Self::Bar,
            125 => Self::RightBrace,
            126 => Self::Tilde,
            127 => Self::Delete,
            256 => Self::Kp0,
            257 => Self::Kp1,
            258 => Self::Kp2,
            259 => Self::Kp3,
            260 => Self::Kp4,
            261 => Self::Kp5,
            262 => Self::Kp6,
            263 => Self::Kp7,
            264 => Self::Kp8,
            265 => Self::Kp9,
            266 => Self::KpPeriod,
            267 => Self::KpDivide,
            268 => Self::KpMultiply,
            269 => Self::KpMinus,
            270 => Self::KpPlus,
            271 => Self::KpEnter,
            272 => Self::KpEquals,
            273 => Self::Up,
            274 => Self::Down,
            275 => Self::Right,
            276 => Self::Left,
            277 => Self::Insert,
            278 => Self::Home,
            279 => Self::End,
            280 => Self::PageUp,
            281 => Self::PageDown,
            282 => Self::F1,
            283 => Self::F2,
            284 => Self::F3,
            285 => Self::F4,
            286 => Self::F5,
            287 => Self::F6,
            288 => Self::F7,
            289 => Self::F8,
            290 => Self::F9,
            291 => Self::F10,
            292 => Self::F11,
            293 => Self::F12,
            294 => Self::F13,
            295 => Self::F14,
            296 => Self::F15,
            300 => Self::NumLock,
            301 => Self::CapsLock,
            302 => Self::ScrollLock,
            303 => Self::RightShift,
            304 => Self::LeftShift,
            305 => Self::RightCtrl,
            306 => Self::LeftCtrl,
            307 => Self::RightAlt,
            308 => Self::LeftAlt,
            309 => Self::RightMeta,
            310 => Self::LeftMeta,
            311 => Self::LeftSuper,
            312 => Self::RightSuper,
            313 => Self::Mode,
            314 => Self::Compose,
            315 => Self::Help,
            316 => Self::Print,
            317 => Self::SysReq,
            318 => Self::Break,
            319 => Self::Menu,
            320 => Self::Power,
            321 => Self::Euro,
            322 => Self::Undo,
            323 => Self::Oem102,
            324 => Self::BrowserBack,
            325 => Self::BrowserForward,
            326 => Self::BrowserRefresh,
            327 => Self::BrowserStop,
            328 => Self::BrowserSearch,
            329 => Self::BrowserFavorites,
            330 => Self::BrowserHome,
            331 => Self::VolumeMute,
            332 => Self::VolumeDown,
            333 => Self::VolumeUp,
            334 => Self::MediaNext,
            335 => Self::MediaPrev,
            336 => Self::MediaStop,
            337 => Self::MediaPlayPause,
            338 => Self::LaunchMail,
            339 => Self::LaunchMedia,
            340 => Self::LaunchApp1,
            341 => Self::LaunchApp2,
            342 => Self::Last,
            other => Self::UnknownKeycode(other),
        }
    }

    pub const fn as_raw(self) -> u32 {
        match self {
            Self::Unknown => 0,
            Self::Backspace => 8,
            Self::Tab => 9,
            Self::Clear => 12,
            Self::Return => 13,
            Self::Pause => 19,
            Self::Escape => 27,
            Self::Space => 32,
            Self::Exclaim => 33,
            Self::QuoteDbl => 34,
            Self::Hash => 35,
            Self::Dollar => 36,
            Self::Ampersand => 38,
            Self::Quote => 39,
            Self::LeftParen => 40,
            Self::RightParen => 41,
            Self::Asterisk => 42,
            Self::Plus => 43,
            Self::Comma => 44,
            Self::Minus => 45,
            Self::Period => 46,
            Self::Slash => 47,
            Self::Num0 => 48,
            Self::Num1 => 49,
            Self::Num2 => 50,
            Self::Num3 => 51,
            Self::Num4 => 52,
            Self::Num5 => 53,
            Self::Num6 => 54,
            Self::Num7 => 55,
            Self::Num8 => 56,
            Self::Num9 => 57,
            Self::Colon => 58,
            Self::Semicolon => 59,
            Self::Less => 60,
            Self::Equals => 61,
            Self::Greater => 62,
            Self::Question => 63,
            Self::At => 64,
            Self::LeftBracket => 91,
            Self::Backslash => 92,
            Self::RightBracket => 93,
            Self::Caret => 94,
            Self::Underscore => 95,
            Self::Backquote => 96,
            Self::A => 97,
            Self::B => 98,
            Self::C => 99,
            Self::D => 100,
            Self::E => 101,
            Self::F => 102,
            Self::G => 103,
            Self::H => 104,
            Self::I => 105,
            Self::J => 106,
            Self::K => 107,
            Self::L => 108,
            Self::M => 109,
            Self::N => 110,
            Self::O => 111,
            Self::P => 112,
            Self::Q => 113,
            Self::R => 114,
            Self::S => 115,
            Self::T => 116,
            Self::U => 117,
            Self::V => 118,
            Self::W => 119,
            Self::X => 120,
            Self::Y => 121,
            Self::Z => 122,
            Self::LeftBrace => 123,
            Self::Bar => 124,
            Self::RightBrace => 125,
            Self::Tilde => 126,
            Self::Delete => 127,
            Self::Kp0 => 256,
            Self::Kp1 => 257,
            Self::Kp2 => 258,
            Self::Kp3 => 259,
            Self::Kp4 => 260,
            Self::Kp5 => 261,
            Self::Kp6 => 262,
            Self::Kp7 => 263,
            Self::Kp8 => 264,
            Self::Kp9 => 265,
            Self::KpPeriod => 266,
            Self::KpDivide => 267,
            Self::KpMultiply => 268,
            Self::KpMinus => 269,
            Self::KpPlus => 270,
            Self::KpEnter => 271,
            Self::KpEquals => 272,
            Self::Up => 273,
            Self::Down => 274,
            Self::Right => 275,
            Self::Left => 276,
            Self::Insert => 277,
            Self::Home => 278,
            Self::End => 279,
            Self::PageUp => 280,
            Self::PageDown => 281,
            Self::F1 => 282,
            Self::F2 => 283,
            Self::F3 => 284,
            Self::F4 => 285,
            Self::F5 => 286,
            Self::F6 => 287,
            Self::F7 => 288,
            Self::F8 => 289,
            Self::F9 => 290,
            Self::F10 => 291,
            Self::F11 => 292,
            Self::F12 => 293,
            Self::F13 => 294,
            Self::F14 => 295,
            Self::F15 => 296,
            Self::NumLock => 300,
            Self::CapsLock => 301,
            Self::ScrollLock => 302,
            Self::RightShift => 303,
            Self::LeftShift => 304,
            Self::RightCtrl => 305,
            Self::LeftCtrl => 306,
            Self::RightAlt => 307,
            Self::LeftAlt => 308,
            Self::RightMeta => 309,
            Self::LeftMeta => 310,
            Self::LeftSuper => 311,
            Self::RightSuper => 312,
            Self::Mode => 313,
            Self::Compose => 314,
            Self::Help => 315,
            Self::Print => 316,
            Self::SysReq => 317,
            Self::Break => 318,
            Self::Menu => 319,
            Self::Power => 320,
            Self::Euro => 321,
            Self::Undo => 322,
            Self::Oem102 => 323,
            Self::BrowserBack => 324,
            Self::BrowserForward => 325,
            Self::BrowserRefresh => 326,
            Self::BrowserStop => 327,
            Self::BrowserSearch => 328,
            Self::BrowserFavorites => 329,
            Self::BrowserHome => 330,
            Self::VolumeMute => 331,
            Self::VolumeDown => 332,
            Self::VolumeUp => 333,
            Self::MediaNext => 334,
            Self::MediaPrev => 335,
            Self::MediaStop => 336,
            Self::MediaPlayPause => 337,
            Self::LaunchMail => 338,
            Self::LaunchMedia => 339,
            Self::LaunchApp1 => 340,
            Self::LaunchApp2 => 341,
            Self::Last => 342,
            Self::UnknownKeycode(keycode) => keycode,
        }
    }
}

#[bitflags]
#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KeyboardModifier {
    Shift = 0x01,
    Ctrl = 0x02,
    Alt = 0x04,
    Meta = 0x08,
    NumLock = 0x10,
    CapsLock = 0x20,
    ScrollLock = 0x40,
}

pub type KeyboardModifiers = BitFlags<KeyboardModifier>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct KeyboardCharacter(u32);

impl KeyboardCharacter {
    pub const fn from_utf32(character: u32) -> Self {
        Self(character)
    }

    pub const fn as_raw(self) -> u32 {
        self.0
    }

    pub fn as_char(self) -> Option<char> {
        char::from_u32(self.0)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct KeyboardEvent {
    pub down: bool,
    pub key: KeyboardKey,
    pub character: KeyboardCharacter,
    pub modifiers: KeyboardModifiers,
}

impl KeyboardEvent {
    pub const fn new(
        down: bool,
        key: KeyboardKey,
        character: KeyboardCharacter,
        modifiers: KeyboardModifiers,
    ) -> Self {
        Self {
            down,
            key,
            character,
            modifiers,
        }
    }

    pub fn from_raw(down: bool, keycode: u32, character: u32, modifiers: u16) -> Self {
        Self::new(
            down,
            KeyboardKey::from_raw(keycode),
            KeyboardCharacter::from_utf32(character),
            KeyboardModifiers::from_bits_truncate(modifiers),
        )
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct InputPort(u32);

impl InputPort {
    pub const fn new(port: u32) -> Self {
        Self(port)
    }

    pub const fn as_raw(self) -> u32 {
        self.0
    }
}

impl From<u32> for InputPort {
    fn from(port: u32) -> Self {
        Self::new(port)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct InputDescriptorIndex(u32);

impl InputDescriptorIndex {
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    pub const fn zero() -> Self {
        Self(0)
    }

    pub const fn as_raw(self) -> u32 {
        self.0
    }
}

impl From<u32> for InputDescriptorIndex {
    fn from(index: u32) -> Self {
        Self::new(index)
    }
}

impl From<AnalogStick> for InputDescriptorIndex {
    fn from(stick: AnalogStick) -> Self {
        Self::new(stick.as_raw())
    }
}

impl From<PointerIndex> for InputDescriptorIndex {
    fn from(index: PointerIndex) -> Self {
        Self::new(index.as_raw())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct InputDescriptorId(u32);

impl InputDescriptorId {
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    pub const fn as_raw(self) -> u32 {
        self.0
    }
}

impl From<u32> for InputDescriptorId {
    fn from(id: u32) -> Self {
        Self::new(id)
    }
}

impl From<JoypadButton> for InputDescriptorId {
    fn from(button: JoypadButton) -> Self {
        Self::new(button.as_raw())
    }
}

impl From<AnalogAxis> for InputDescriptorId {
    fn from(axis: AnalogAxis) -> Self {
        Self::new(axis.as_raw())
    }
}

impl From<MouseAxis> for InputDescriptorId {
    fn from(axis: MouseAxis) -> Self {
        Self::new(axis.as_raw())
    }
}

impl From<MouseButton> for InputDescriptorId {
    fn from(button: MouseButton) -> Self {
        Self::new(button.as_raw())
    }
}

impl From<MouseWheel> for InputDescriptorId {
    fn from(wheel: MouseWheel) -> Self {
        Self::new(wheel.as_raw())
    }
}

impl From<PointerAxis> for InputDescriptorId {
    fn from(axis: PointerAxis) -> Self {
        Self::new(axis.as_raw())
    }
}

impl From<LightgunAxis> for InputDescriptorId {
    fn from(axis: LightgunAxis) -> Self {
        Self::new(axis.as_raw())
    }
}

impl From<LightgunButton> for InputDescriptorId {
    fn from(button: LightgunButton) -> Self {
        Self::new(button.as_raw())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputDescriptor {
    pub port: InputPort,
    pub device: ControllerDevice,
    pub index: InputDescriptorIndex,
    pub id: InputDescriptorId,
    pub description: String,
}

impl InputDescriptor {
    pub fn new(
        port: impl Into<InputPort>,
        device: ControllerDevice,
        index: impl Into<InputDescriptorIndex>,
        id: impl Into<InputDescriptorId>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            port: port.into(),
            device,
            index: index.into(),
            id: id.into(),
            description: description.into(),
        }
    }

    pub fn joypad(
        port: impl Into<InputPort>,
        button: JoypadButton,
        description: impl Into<String>,
    ) -> Self {
        Self::new(
            port,
            ControllerDevice::Joypad,
            InputDescriptorIndex::zero(),
            button,
            description,
        )
    }

    pub fn analog(
        port: impl Into<InputPort>,
        stick: AnalogStick,
        axis: AnalogAxis,
        description: impl Into<String>,
    ) -> Self {
        Self::new(port, ControllerDevice::Analog, stick, axis, description)
    }

    pub fn mouse(
        port: impl Into<InputPort>,
        id: impl Into<InputDescriptorId>,
        description: impl Into<String>,
    ) -> Self {
        Self::new(
            port,
            ControllerDevice::Mouse,
            InputDescriptorIndex::zero(),
            id,
            description,
        )
    }

    pub fn pointer(
        port: impl Into<InputPort>,
        index: impl Into<InputDescriptorIndex>,
        id: impl Into<InputDescriptorId>,
        description: impl Into<String>,
    ) -> Self {
        Self::new(port, ControllerDevice::Pointer, index, id, description)
    }

    pub fn lightgun(
        port: impl Into<InputPort>,
        id: impl Into<InputDescriptorId>,
        description: impl Into<String>,
    ) -> Self {
        Self::new(
            port,
            ControllerDevice::Lightgun,
            InputDescriptorIndex::zero(),
            id,
            description,
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ControllerDescription {
    pub description: String,
    pub device: ControllerDevice,
}

impl ControllerDescription {
    pub fn new(description: impl Into<String>, device: ControllerDevice) -> Self {
        Self {
            description: description.into(),
            device,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ControllerInfo {
    pub types: Vec<ControllerDescription>,
}

impl ControllerInfo {
    pub fn new(types: impl Into<Vec<ControllerDescription>>) -> Self {
        Self {
            types: types.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PointerIndex(u32);

impl PointerIndex {
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    pub const fn as_raw(self) -> u32 {
        self.0
    }
}

impl From<u32> for PointerIndex {
    fn from(index: u32) -> Self {
        Self::new(index)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ControllerDevice {
    None,
    Joypad,
    Mouse,
    Keyboard,
    Lightgun,
    Analog,
    Pointer,
    Subclass(ControllerDeviceSubclass),
    Unknown(u32),
}

impl ControllerDevice {
    pub const fn from_raw(device: u32) -> Self {
        let base = device & RETRO_DEVICE_MASK;
        if device != base {
            return match Self::from_base_raw(base) {
                Some(_) => Self::Subclass(ControllerDeviceSubclass(device)),
                None => Self::Unknown(device),
            };
        }
        match Self::from_base_raw(device) {
            Some(device) => device,
            None => Self::Unknown(device),
        }
    }

    const fn from_base_raw(device: u32) -> Option<Self> {
        match device {
            RETRO_DEVICE_NONE => Some(Self::None),
            RETRO_DEVICE_JOYPAD => Some(Self::Joypad),
            RETRO_DEVICE_MOUSE => Some(Self::Mouse),
            RETRO_DEVICE_KEYBOARD => Some(Self::Keyboard),
            RETRO_DEVICE_LIGHTGUN => Some(Self::Lightgun),
            RETRO_DEVICE_ANALOG => Some(Self::Analog),
            RETRO_DEVICE_POINTER => Some(Self::Pointer),
            _ => None,
        }
    }

    pub const fn as_raw(self) -> u32 {
        match self {
            Self::None => RETRO_DEVICE_NONE,
            Self::Joypad => RETRO_DEVICE_JOYPAD,
            Self::Mouse => RETRO_DEVICE_MOUSE,
            Self::Keyboard => RETRO_DEVICE_KEYBOARD,
            Self::Lightgun => RETRO_DEVICE_LIGHTGUN,
            Self::Analog => RETRO_DEVICE_ANALOG,
            Self::Pointer => RETRO_DEVICE_POINTER,
            Self::Subclass(device) => device.as_raw(),
            Self::Unknown(device) => device,
        }
    }

    pub const fn base_device(self) -> Self {
        match self {
            Self::Subclass(device) => device.base_device(),
            Self::Unknown(device) => match Self::from_base_raw(device & RETRO_DEVICE_MASK) {
                Some(base) => base,
                None => Self::Unknown(device & RETRO_DEVICE_MASK),
            },
            base => base,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ControllerDeviceSubclass(u32);

impl ControllerDeviceSubclass {
    pub const fn new(base: ControllerDevice, id: u32) -> Option<Self> {
        let base = base.base_device().as_raw();
        if base > RETRO_DEVICE_MASK {
            return None;
        }
        match id.checked_add(1) {
            Some(id) if id <= (u32::MAX >> RETRO_DEVICE_TYPE_SHIFT) => {
                Some(Self((id << RETRO_DEVICE_TYPE_SHIFT) | base))
            }
            None => None,
            _ => None,
        }
    }

    pub const fn from_raw(device: u32) -> Option<Self> {
        let base = device & RETRO_DEVICE_MASK;
        if device == base || ControllerDevice::from_base_raw(base).is_none() {
            return None;
        }
        Some(Self(device))
    }

    pub const fn as_raw(self) -> u32 {
        self.0
    }

    pub const fn base_device(self) -> ControllerDevice {
        match ControllerDevice::from_base_raw(self.0 & RETRO_DEVICE_MASK) {
            Some(device) => device,
            None => ControllerDevice::Unknown(self.0 & RETRO_DEVICE_MASK),
        }
    }

    pub const fn id(self) -> u32 {
        (self.0 >> RETRO_DEVICE_TYPE_SHIFT) - 1
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JoypadButton {
    B,
    Y,
    Select,
    Start,
    Up,
    Down,
    Left,
    Right,
    A,
    X,
    L,
    R,
    L2,
    R2,
    L3,
    R3,
}

impl JoypadButton {
    pub(crate) const fn as_raw(self) -> u32 {
        match self {
            Self::B => RETRO_DEVICE_ID_JOYPAD_B,
            Self::Y => RETRO_DEVICE_ID_JOYPAD_Y,
            Self::Select => RETRO_DEVICE_ID_JOYPAD_SELECT,
            Self::Start => RETRO_DEVICE_ID_JOYPAD_START,
            Self::Up => RETRO_DEVICE_ID_JOYPAD_UP,
            Self::Down => RETRO_DEVICE_ID_JOYPAD_DOWN,
            Self::Left => RETRO_DEVICE_ID_JOYPAD_LEFT,
            Self::Right => RETRO_DEVICE_ID_JOYPAD_RIGHT,
            Self::A => RETRO_DEVICE_ID_JOYPAD_A,
            Self::X => RETRO_DEVICE_ID_JOYPAD_X,
            Self::L => RETRO_DEVICE_ID_JOYPAD_L,
            Self::R => RETRO_DEVICE_ID_JOYPAD_R,
            Self::L2 => RETRO_DEVICE_ID_JOYPAD_L2,
            Self::R2 => RETRO_DEVICE_ID_JOYPAD_R2,
            Self::L3 => RETRO_DEVICE_ID_JOYPAD_L3,
            Self::R3 => RETRO_DEVICE_ID_JOYPAD_R3,
        }
    }

    const fn mask(self) -> u16 {
        1u16 << self.as_raw()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct JoypadButtonSet(u16);

impl JoypadButtonSet {
    pub const fn from_raw_bits(bits: u16) -> Self {
        Self(bits)
    }

    pub const fn raw_bits(self) -> u16 {
        self.0
    }

    pub const fn contains(self, button: JoypadButton) -> bool {
        self.0 & button.mask() != 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AnalogStick {
    Left,
    Right,
}

impl AnalogStick {
    pub(crate) const fn as_raw(self) -> u32 {
        match self {
            Self::Left => RETRO_DEVICE_INDEX_ANALOG_LEFT,
            Self::Right => RETRO_DEVICE_INDEX_ANALOG_RIGHT,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AnalogAxis {
    X,
    Y,
}

impl AnalogAxis {
    pub(crate) const fn as_raw(self) -> u32 {
        match self {
            Self::X => RETRO_DEVICE_ID_ANALOG_X,
            Self::Y => RETRO_DEVICE_ID_ANALOG_Y,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MouseAxis {
    X,
    Y,
}

impl MouseAxis {
    pub(crate) const fn as_raw(self) -> u32 {
        match self {
            Self::X => RETRO_DEVICE_ID_MOUSE_X,
            Self::Y => RETRO_DEVICE_ID_MOUSE_Y,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Button4,
    Button5,
}

impl MouseButton {
    pub(crate) const fn as_raw(self) -> u32 {
        match self {
            Self::Left => RETRO_DEVICE_ID_MOUSE_LEFT,
            Self::Right => RETRO_DEVICE_ID_MOUSE_RIGHT,
            Self::Middle => RETRO_DEVICE_ID_MOUSE_MIDDLE,
            Self::Button4 => RETRO_DEVICE_ID_MOUSE_BUTTON_4,
            Self::Button5 => RETRO_DEVICE_ID_MOUSE_BUTTON_5,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MouseWheel {
    Up,
    Down,
    Left,
    Right,
}

impl MouseWheel {
    pub(crate) const fn as_raw(self) -> u32 {
        match self {
            Self::Up => RETRO_DEVICE_ID_MOUSE_WHEELUP,
            Self::Down => RETRO_DEVICE_ID_MOUSE_WHEELDOWN,
            Self::Left => RETRO_DEVICE_ID_MOUSE_HORIZ_WHEELDOWN,
            Self::Right => RETRO_DEVICE_ID_MOUSE_HORIZ_WHEELUP,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PointerAxis {
    X,
    Y,
}

impl PointerAxis {
    pub(crate) const fn as_raw(self) -> u32 {
        match self {
            Self::X => RETRO_DEVICE_ID_POINTER_X,
            Self::Y => RETRO_DEVICE_ID_POINTER_Y,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LightgunAxis {
    #[deprecated(note = "use ScreenX for the modern absolute lightgun X coordinate")]
    RelativeX,
    #[deprecated(note = "use ScreenY for the modern absolute lightgun Y coordinate")]
    RelativeY,
    ScreenX,
    ScreenY,
}

impl LightgunAxis {
    #[allow(deprecated)]
    pub(crate) const fn as_raw(self) -> u32 {
        match self {
            Self::RelativeX => RETRO_DEVICE_ID_LIGHTGUN_X,
            Self::RelativeY => RETRO_DEVICE_ID_LIGHTGUN_Y,
            Self::ScreenX => RETRO_DEVICE_ID_LIGHTGUN_SCREEN_X,
            Self::ScreenY => RETRO_DEVICE_ID_LIGHTGUN_SCREEN_Y,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LightgunButton {
    Trigger,
    Reload,
    AuxA,
    AuxB,
    AuxC,
    Start,
    Select,
    DpadUp,
    DpadDown,
    DpadLeft,
    DpadRight,
    #[deprecated(note = "use AuxA instead")]
    Cursor,
    #[deprecated(note = "use AuxB instead")]
    Turbo,
    #[deprecated(note = "use Start instead")]
    Pause,
}

impl LightgunButton {
    #[allow(deprecated)]
    pub(crate) const fn as_raw(self) -> u32 {
        match self {
            Self::Trigger => RETRO_DEVICE_ID_LIGHTGUN_TRIGGER,
            Self::Reload => RETRO_DEVICE_ID_LIGHTGUN_RELOAD,
            Self::AuxA => RETRO_DEVICE_ID_LIGHTGUN_AUX_A,
            Self::AuxB => RETRO_DEVICE_ID_LIGHTGUN_AUX_B,
            Self::AuxC => RETRO_DEVICE_ID_LIGHTGUN_AUX_C,
            Self::Start => RETRO_DEVICE_ID_LIGHTGUN_START,
            Self::Select => RETRO_DEVICE_ID_LIGHTGUN_SELECT,
            Self::DpadUp => RETRO_DEVICE_ID_LIGHTGUN_DPAD_UP,
            Self::DpadDown => RETRO_DEVICE_ID_LIGHTGUN_DPAD_DOWN,
            Self::DpadLeft => RETRO_DEVICE_ID_LIGHTGUN_DPAD_LEFT,
            Self::DpadRight => RETRO_DEVICE_ID_LIGHTGUN_DPAD_RIGHT,
            Self::Cursor => crate::raw::RETRO_DEVICE_ID_LIGHTGUN_CURSOR,
            Self::Turbo => crate::raw::RETRO_DEVICE_ID_LIGHTGUN_TURBO,
            Self::Pause => crate::raw::RETRO_DEVICE_ID_LIGHTGUN_PAUSE,
        }
    }
}

pub(crate) const fn joypad_mask_query_id() -> u32 {
    RETRO_DEVICE_ID_JOYPAD_MASK
}

pub(crate) const fn analog_button_index() -> u32 {
    RETRO_DEVICE_INDEX_ANALOG_BUTTON
}

pub(crate) const fn pointer_pressed_id() -> u32 {
    RETRO_DEVICE_ID_POINTER_PRESSED
}

pub(crate) const fn pointer_count_id() -> u32 {
    RETRO_DEVICE_ID_POINTER_COUNT
}

pub(crate) const fn pointer_is_offscreen_id() -> u32 {
    RETRO_DEVICE_ID_POINTER_IS_OFFSCREEN
}

pub(crate) const fn lightgun_is_offscreen_id() -> u32 {
    RETRO_DEVICE_ID_LIGHTGUN_IS_OFFSCREEN
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn controller_devices_round_trip_to_libretro_ids() {
        let devices = [
            ControllerDevice::None,
            ControllerDevice::Joypad,
            ControllerDevice::Mouse,
            ControllerDevice::Keyboard,
            ControllerDevice::Lightgun,
            ControllerDevice::Analog,
            ControllerDevice::Pointer,
        ];

        for device in devices {
            assert_eq!(ControllerDevice::from_raw(device.as_raw()), device);
        }
    }

    #[test]
    fn led_state_encodes_bool_like_values() {
        assert_eq!(LedState::Off.as_raw(), 0);
        assert_eq!(LedState::On.as_raw(), 1);
        assert_eq!(LedIndex::new(2).as_raw(), 2);
    }

    #[test]
    fn empty_led_interface_reports_unavailable() {
        let leds = LedInterface::default();

        assert!(!leds.is_available());
        assert!(!leds.set_state(0, LedState::On));
    }

    #[test]
    fn rumble_values_encode_libretro_state() {
        assert_eq!(
            RumbleEffect::Strong.as_raw(),
            crate::raw::retro_rumble_effect::Strong
        );
        assert_eq!(
            RumbleEffect::Weak.as_raw(),
            crate::raw::retro_rumble_effect::Weak
        );
        assert_eq!(RumbleStrength::off().as_raw(), 0);
        assert_eq!(RumbleStrength::max().as_raw(), u16::MAX);
        assert_eq!(RumbleStrength::new(123).as_raw(), 123);
    }

    #[test]
    fn empty_rumble_interface_reports_unavailable() {
        let rumble = RumbleInterface::default();

        assert!(!rumble.is_available());
        assert!(!rumble.set_state(0, RumbleEffect::Strong, RumbleStrength::max()));
    }

    #[test]
    fn unknown_controller_device_preserves_original_id() {
        assert_eq!(
            ControllerDevice::from_raw(123),
            ControllerDevice::Unknown(123)
        );
        assert_eq!(ControllerDevice::Unknown(123).as_raw(), 123);
    }

    #[test]
    fn controller_device_subclasses_preserve_base_device_and_id() {
        let lightgun_scope = ControllerDeviceSubclass::new(ControllerDevice::Lightgun, 1)
            .expect("lightgun subclass should be representable");

        assert_eq!(lightgun_scope.base_device(), ControllerDevice::Lightgun);
        assert_eq!(lightgun_scope.id(), 1);
        assert_eq!(
            ControllerDevice::from_raw(lightgun_scope.as_raw()),
            ControllerDevice::Subclass(lightgun_scope)
        );
        assert_eq!(
            ControllerDevice::Subclass(lightgun_scope).base_device(),
            ControllerDevice::Lightgun
        );
        assert_eq!(
            lightgun_scope.as_raw(),
            ((1 + 1) << crate::raw::RETRO_DEVICE_TYPE_SHIFT) | crate::raw::RETRO_DEVICE_LIGHTGUN
        );
        assert_eq!(
            ControllerDeviceSubclass::from_raw(ControllerDevice::Mouse.as_raw()),
            None
        );
    }

    #[test]
    fn input_device_capabilities_map_to_controller_devices() {
        assert_eq!(
            InputDeviceCapability::Joypad.device(),
            ControllerDevice::Joypad
        );
        assert_eq!(
            InputDeviceCapability::Mouse.device(),
            ControllerDevice::Mouse
        );
        assert_eq!(
            InputDeviceCapability::Keyboard.device(),
            ControllerDevice::Keyboard
        );
        assert_eq!(
            InputDeviceCapability::Lightgun.device(),
            ControllerDevice::Lightgun
        );
        assert_eq!(
            InputDeviceCapability::Analog.device(),
            ControllerDevice::Analog
        );
        assert_eq!(
            InputDeviceCapability::Pointer.device(),
            ControllerDevice::Pointer
        );
    }

    #[test]
    fn keyboard_keys_round_trip_and_preserve_unknown_keycodes() {
        assert_eq!(KeyboardKey::from_raw(97), KeyboardKey::A);
        assert_eq!(KeyboardKey::A.as_raw(), 97);
        assert_eq!(KeyboardKey::from_raw(282), KeyboardKey::F1);
        assert_eq!(KeyboardKey::F1.as_raw(), 282);
        assert_eq!(
            KeyboardKey::from_raw(65_535),
            KeyboardKey::UnknownKeycode(65_535)
        );
        assert_eq!(KeyboardKey::UnknownKeycode(65_535).as_raw(), 65_535);
    }

    #[test]
    fn keyboard_event_preserves_text_and_modifiers() {
        let event = KeyboardEvent::from_raw(
            true,
            KeyboardKey::Return.as_raw(),
            0x00e9,
            (KeyboardModifiers::from(KeyboardModifier::Shift) | KeyboardModifier::Ctrl).bits(),
        );

        assert!(event.down);
        assert_eq!(event.key, KeyboardKey::Return);
        assert_eq!(event.character.as_char(), char::from_u32(0x00e9));
        assert!(event.modifiers.contains(KeyboardModifier::Shift));
        assert!(event.modifiers.contains(KeyboardModifier::Ctrl));
        assert!(!event.modifiers.contains(KeyboardModifier::Alt));
    }

    #[test]
    fn input_descriptor_builders_use_typed_device_ids() {
        let joypad = InputDescriptor::joypad(0, JoypadButton::A, "Jump");
        assert_eq!(joypad.port, InputPort::new(0));
        assert_eq!(joypad.device, ControllerDevice::Joypad);
        assert_eq!(joypad.index, InputDescriptorIndex::zero());
        assert_eq!(joypad.id, InputDescriptorId::from(JoypadButton::A));

        let analog = InputDescriptor::analog(1, AnalogStick::Right, AnalogAxis::Y, "Look up");
        assert_eq!(analog.port, InputPort::new(1));
        assert_eq!(analog.device, ControllerDevice::Analog);
        assert_eq!(analog.index, InputDescriptorIndex::from(AnalogStick::Right));
        assert_eq!(analog.id, InputDescriptorId::from(AnalogAxis::Y));
    }

    #[test]
    fn joypad_button_set_uses_libretro_mask_bits() {
        let buttons =
            JoypadButtonSet::from_raw_bits(JoypadButton::A.mask() | JoypadButton::L.mask());

        assert!(buttons.contains(JoypadButton::A));
        assert!(buttons.contains(JoypadButton::L));
        assert!(!buttons.contains(JoypadButton::B));
    }

    #[test]
    #[allow(deprecated)]
    fn deprecated_lightgun_aliases_map_to_their_modern_buttons() {
        assert_eq!(
            crate::raw::RETRO_DEVICE_ID_LIGHTGUN_CURSOR,
            crate::raw::RETRO_DEVICE_ID_LIGHTGUN_AUX_A
        );
        assert_eq!(
            crate::raw::RETRO_DEVICE_ID_LIGHTGUN_TURBO,
            crate::raw::RETRO_DEVICE_ID_LIGHTGUN_AUX_B
        );
        assert_eq!(
            LightgunButton::AuxA.as_raw(),
            crate::raw::RETRO_DEVICE_ID_LIGHTGUN_CURSOR
        );
        assert_eq!(
            LightgunButton::Cursor.as_raw(),
            LightgunButton::AuxA.as_raw()
        );
        assert_eq!(
            LightgunButton::Turbo.as_raw(),
            LightgunButton::AuxB.as_raw()
        );
        assert_eq!(
            LightgunButton::Pause.as_raw(),
            crate::raw::RETRO_DEVICE_ID_LIGHTGUN_PAUSE
        );
    }

    #[test]
    #[allow(deprecated)]
    fn deprecated_lightgun_relative_axes_are_typed() {
        assert_eq!(
            LightgunAxis::RelativeX.as_raw(),
            crate::raw::RETRO_DEVICE_ID_LIGHTGUN_X
        );
        assert_eq!(
            LightgunAxis::RelativeY.as_raw(),
            crate::raw::RETRO_DEVICE_ID_LIGHTGUN_Y
        );
    }
}
