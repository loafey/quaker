use bevy::ecs::system::Resource;
use input_derive::derive_input;
use serde::Deserialize;

#[derive_input]
#[derive(Debug, Deserialize, Resource)]
#[allow(dead_code)]
pub struct PlayerInput {
    weapon_shoot1: Key,
    weapon_shoot2: Key,
    weapon_next: Key,
    weapon_previous: Key,
    walk_forward: Key,
    walk_backward: Key,
    walk_left: Key,
    walk_right: Key,
    jump: Key,
    debug_fly_up: Key,
    debug_fly_down: Key,
    pause_game: Key,
    pause_game_alt: Key,
}
impl PlayerInput {
    pub fn new() -> Self {
        serde_json::from_str(&std::fs::read_to_string("assets/inputs.json").unwrap()).unwrap()
    }
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(untagged)]
pub enum Key {
    Mouse(MouseKey),
    Wheel(MouseWheel),
    Keyboard(KeyCode),
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub enum MouseWheel {
    Wheel1,
    Wheel2,
}
impl MouseWheel {
    #[allow(dead_code)]
    fn check(self, dir: f32) -> bool {
        match self {
            MouseWheel::Wheel1 => dir > 0.0,
            MouseWheel::Wheel2 => dir < 0.0,
        }
    }
}
#[derive(Debug, Deserialize, Copy, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum MouseKey {
    MouseLeft,
    MouseRight,
    MouseMiddle,
    MouseBack,
    MouseForward,
    MouseOther(u16),
}

#[derive(Debug, Deserialize, Copy, Clone)]
#[repr(u32)]
pub enum KeyCode {
    Unidentified((u16, u32)),
    Backquote,
    Backslash,
    BracketLeft,
    BracketRight,
    Comma,
    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    Equal,
    IntlBackslash,
    IntlRo,
    IntlYen,
    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
    KeyF,
    KeyG,
    KeyH,
    KeyI,
    KeyJ,
    KeyK,
    KeyL,
    KeyM,
    KeyN,
    KeyO,
    KeyP,
    KeyQ,
    KeyR,
    KeyS,
    KeyT,
    KeyU,
    KeyV,
    KeyW,
    KeyX,
    KeyY,
    KeyZ,
    Minus,
    Period,
    Quote,
    Semicolon,
    Slash,
    AltLeft,
    AltRight,
    Backspace,
    CapsLock,
    ContextMenu,
    ControlLeft,
    ControlRight,
    Enter,
    SuperLeft,
    SuperRight,
    ShiftLeft,
    ShiftRight,
    Space,
    Tab,
    Convert,
    KanaMode,
    Lang1,
    Lang2,
    Lang3,
    Lang4,
    Lang5,
    NonConvert,
    Delete,
    End,
    Help,
    Home,
    Insert,
    PageDown,
    PageUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    NumLock,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadAdd,
    NumpadBackspace,
    NumpadClear,
    NumpadClearEntry,
    NumpadComma,
    NumpadDecimal,
    NumpadDivide,
    NumpadEnter,
    NumpadEqual,
    NumpadHash,
    NumpadMemoryAdd,
    NumpadMemoryClear,
    NumpadMemoryRecall,
    NumpadMemoryStore,
    NumpadMemorySubtract,
    NumpadMultiply,
    NumpadParenLeft,
    NumpadParenRight,
    NumpadStar,
    NumpadSubtract,
    Escape,
    Fn,
    FnLock,
    PrintScreen,
    ScrollLock,
    Pause,
    BrowserBack,
    BrowserFavorites,
    BrowserForward,
    BrowserHome,
    BrowserRefresh,
    BrowserSearch,
    BrowserStop,
    Eject,
    LaunchApp1,
    LaunchApp2,
    LaunchMail,
    MediaPlayPause,
    MediaSelect,
    MediaStop,
    MediaTrackNext,
    MediaTrackPrevious,
    Power,
    Sleep,
    AudioVolumeDown,
    AudioVolumeMute,
    AudioVolumeUp,
    WakeUp,
    Meta,
    Hyper,
    Turbo,
    Abort,
    Resume,
    Suspend,
    Again,
    Copy,
    Cut,
    Find,
    Open,
    Paste,
    Props,
    Select,
    Undo,
    Hiragana,
    Katakana,
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
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    F25,
    F26,
    F27,
    F28,
    F29,
    F30,
    F31,
    F32,
    F33,
    F34,
    F35,
}
