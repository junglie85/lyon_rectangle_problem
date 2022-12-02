use glam::{Vec2, Vec4, Vec4Swizzles};
use winit::event::VirtualKeyCode;
use winit_input_helper::WinitInputHelper;

use crate::camera::Camera;

pub struct InputHelper<'frame> {
    winit_helper: &'frame WinitInputHelper,
}

impl<'frame> InputHelper<'frame> {
    pub(crate) fn new(winit_helper: &'frame WinitInputHelper) -> Self {
        Self { winit_helper }
    }

    pub fn quit(&self) -> bool {
        self.winit_helper.quit()
    }

    pub fn key_pressed(&self, key_code: KeyCode) -> bool {
        self.winit_helper.key_pressed(key_code.into())
    }

    pub fn key_released(&self, key_code: KeyCode) -> bool {
        self.winit_helper.key_released(key_code.into())
    }

    pub fn mouse_pressed(&self, button: MouseButton) -> bool {
        self.winit_helper.mouse_pressed(button.into())
    }

    pub fn mouse_released(&self, button: MouseButton) -> bool {
        self.winit_helper.mouse_released(button.into())
    }

    pub fn mouse_in_viewport(&self) -> Vec2 {
        let (x, y) = self.winit_helper.mouse().unwrap_or_default();

        Vec2::new(x, y)
    }

    pub fn mouse_in_world(&self, camera: &Camera) -> Vec2 {
        let viewport_position = self.mouse_in_viewport();
        let viewport_dimensions = Vec2::new(camera.width(), camera.height());
        let mut ndc = viewport_position / viewport_dimensions * 2.0 - 1.0;
        ndc.y *= -1.0;
        let ndc = Vec4::from((ndc, 1.0, 1.0));

        let inverse_projection = camera.get_projection().inverse();
        let inverse_view = camera.get_view().inverse();

        let mouse_position = inverse_view * inverse_projection * ndc;

        mouse_position.xy()
    }
}

// Copied from winit.
#[derive(Debug, Clone, Copy)]
pub enum KeyCode {
    /// The '1' key over the letters.
    Key1,
    /// The '2' key over the letters.
    Key2,
    /// The '3' key over the letters.
    Key3,
    /// The '4' key over the letters.
    Key4,
    /// The '5' key over the letters.
    Key5,
    /// The '6' key over the letters.
    Key6,
    /// The '7' key over the letters.
    Key7,
    /// The '8' key over the letters.
    Key8,
    /// The '9' key over the letters.
    Key9,
    /// The '0' key over the 'O' and 'P' keys.
    Key0,
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
    /// The Escape key, next to F1.
    Escape,
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
    /// Print Screen/SysRq.
    Snapshot,
    /// Scroll Lock.
    Scroll,
    /// Pause/Break key, next to Scroll lock.
    Pause,
    /// `Insert`, next to Backspace.
    Insert,
    Home,
    Delete,
    End,
    PageDown,
    PageUp,
    Left,
    Up,
    Right,
    Down,
    /// The Backspace key, right over Enter.
    Backspace,
    /// The Enter key.
    Return,
    /// The space bar.
    Space,
    /// The "Compose" key on Linux.
    Compose,
    Caret,
    Numlock,
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
    NumpadDivide,
    NumpadDecimal,
    NumpadComma,
    NumpadEnter,
    NumpadEquals,
    NumpadMultiply,
    NumpadSubtract,
    AbntC1,
    AbntC2,
    Apostrophe,
    Apps,
    Asterisk,
    At,
    Ax,
    Backslash,
    Calculator,
    Capital,
    Colon,
    Comma,
    Convert,
    Equals,
    Grave,
    Kana,
    Kanji,
    LAlt,
    LBracket,
    LControl,
    LShift,
    LWin,
    Mail,
    MediaSelect,
    MediaStop,
    Minus,
    Mute,
    MyComputer,
    // also called "Next"
    NavigateForward,
    // also called "Prior"
    NavigateBackward,
    NextTrack,
    NoConvert,
    OEM102,
    Period,
    PlayPause,
    Plus,
    Power,
    PrevTrack,
    RAlt,
    RBracket,
    RControl,
    RShift,
    RWin,
    Semicolon,
    Slash,
    Sleep,
    Stop,
    Sysrq,
    Tab,
    Underline,
    Unlabeled,
    VolumeDown,
    VolumeUp,
    Wake,
    WebBack,
    WebFavorites,
    WebForward,
    WebHome,
    WebRefresh,
    WebSearch,
    WebStop,
    Yen,
    Copy,
    Paste,
    Cut,
}

impl Into<VirtualKeyCode> for KeyCode {
    fn into(self) -> VirtualKeyCode {
        match self {
            KeyCode::Key1 => VirtualKeyCode::Key1,
            KeyCode::Key2 => VirtualKeyCode::Key2,
            KeyCode::Key3 => VirtualKeyCode::Key3,
            KeyCode::Key4 => VirtualKeyCode::Key4,
            KeyCode::Key5 => VirtualKeyCode::Key5,
            KeyCode::Key6 => VirtualKeyCode::Key6,
            KeyCode::Key7 => VirtualKeyCode::Key7,
            KeyCode::Key8 => VirtualKeyCode::Key8,
            KeyCode::Key9 => VirtualKeyCode::Key9,
            KeyCode::Key0 => VirtualKeyCode::Key0,
            KeyCode::A => VirtualKeyCode::A,
            KeyCode::B => VirtualKeyCode::B,
            KeyCode::C => VirtualKeyCode::C,
            KeyCode::D => VirtualKeyCode::D,
            KeyCode::E => VirtualKeyCode::E,
            KeyCode::F => VirtualKeyCode::F,
            KeyCode::G => VirtualKeyCode::G,
            KeyCode::H => VirtualKeyCode::H,
            KeyCode::I => VirtualKeyCode::I,
            KeyCode::J => VirtualKeyCode::J,
            KeyCode::K => VirtualKeyCode::K,
            KeyCode::L => VirtualKeyCode::L,
            KeyCode::M => VirtualKeyCode::M,
            KeyCode::N => VirtualKeyCode::N,
            KeyCode::O => VirtualKeyCode::O,
            KeyCode::P => VirtualKeyCode::P,
            KeyCode::Q => VirtualKeyCode::Q,
            KeyCode::R => VirtualKeyCode::R,
            KeyCode::S => VirtualKeyCode::S,
            KeyCode::T => VirtualKeyCode::T,
            KeyCode::U => VirtualKeyCode::U,
            KeyCode::V => VirtualKeyCode::V,
            KeyCode::W => VirtualKeyCode::W,
            KeyCode::X => VirtualKeyCode::X,
            KeyCode::Y => VirtualKeyCode::Y,
            KeyCode::Z => VirtualKeyCode::Z,
            KeyCode::Escape => VirtualKeyCode::Escape,
            KeyCode::F1 => VirtualKeyCode::F1,
            KeyCode::F2 => VirtualKeyCode::F2,
            KeyCode::F3 => VirtualKeyCode::F3,
            KeyCode::F4 => VirtualKeyCode::F4,
            KeyCode::F5 => VirtualKeyCode::F5,
            KeyCode::F6 => VirtualKeyCode::F6,
            KeyCode::F7 => VirtualKeyCode::F7,
            KeyCode::F8 => VirtualKeyCode::F8,
            KeyCode::F9 => VirtualKeyCode::F9,
            KeyCode::F10 => VirtualKeyCode::F10,
            KeyCode::F11 => VirtualKeyCode::F11,
            KeyCode::F12 => VirtualKeyCode::F12,
            KeyCode::F13 => VirtualKeyCode::F13,
            KeyCode::F14 => VirtualKeyCode::F14,
            KeyCode::F15 => VirtualKeyCode::F15,
            KeyCode::F16 => VirtualKeyCode::F16,
            KeyCode::F17 => VirtualKeyCode::F17,
            KeyCode::F18 => VirtualKeyCode::F18,
            KeyCode::F19 => VirtualKeyCode::F19,
            KeyCode::F20 => VirtualKeyCode::F20,
            KeyCode::F21 => VirtualKeyCode::F21,
            KeyCode::F22 => VirtualKeyCode::F22,
            KeyCode::F23 => VirtualKeyCode::F23,
            KeyCode::F24 => VirtualKeyCode::F24,
            KeyCode::Snapshot => VirtualKeyCode::Snapshot,
            KeyCode::Scroll => VirtualKeyCode::Scroll,
            KeyCode::Pause => VirtualKeyCode::Pause,
            KeyCode::Insert => VirtualKeyCode::Insert,
            KeyCode::Home => VirtualKeyCode::Home,
            KeyCode::Delete => VirtualKeyCode::Delete,
            KeyCode::End => VirtualKeyCode::End,
            KeyCode::PageDown => VirtualKeyCode::PageDown,
            KeyCode::PageUp => VirtualKeyCode::PageUp,
            KeyCode::Left => VirtualKeyCode::Left,
            KeyCode::Up => VirtualKeyCode::Up,
            KeyCode::Right => VirtualKeyCode::Right,
            KeyCode::Down => VirtualKeyCode::Down,
            KeyCode::Backspace => VirtualKeyCode::Back,
            KeyCode::Return => VirtualKeyCode::Return,
            KeyCode::Space => VirtualKeyCode::Space,
            KeyCode::Compose => VirtualKeyCode::Compose,
            KeyCode::Caret => VirtualKeyCode::Caret,
            KeyCode::Numlock => VirtualKeyCode::Numlock,
            KeyCode::Numpad0 => VirtualKeyCode::Numpad0,
            KeyCode::Numpad1 => VirtualKeyCode::Numpad1,
            KeyCode::Numpad2 => VirtualKeyCode::Numpad2,
            KeyCode::Numpad3 => VirtualKeyCode::Numpad3,
            KeyCode::Numpad4 => VirtualKeyCode::Numpad4,
            KeyCode::Numpad5 => VirtualKeyCode::Numpad5,
            KeyCode::Numpad6 => VirtualKeyCode::Numpad6,
            KeyCode::Numpad7 => VirtualKeyCode::Numpad7,
            KeyCode::Numpad8 => VirtualKeyCode::Numpad8,
            KeyCode::Numpad9 => VirtualKeyCode::Numpad9,
            KeyCode::NumpadAdd => VirtualKeyCode::NumpadAdd,
            KeyCode::NumpadDivide => VirtualKeyCode::NumpadDivide,
            KeyCode::NumpadDecimal => VirtualKeyCode::NumpadDecimal,
            KeyCode::NumpadComma => VirtualKeyCode::NumpadComma,
            KeyCode::NumpadEnter => VirtualKeyCode::NumpadEnter,
            KeyCode::NumpadEquals => VirtualKeyCode::NumpadEquals,
            KeyCode::NumpadMultiply => VirtualKeyCode::NumpadMultiply,
            KeyCode::NumpadSubtract => VirtualKeyCode::NumpadSubtract,
            KeyCode::AbntC1 => VirtualKeyCode::AbntC1,
            KeyCode::AbntC2 => VirtualKeyCode::AbntC2,
            KeyCode::Apostrophe => VirtualKeyCode::Apostrophe,
            KeyCode::Apps => VirtualKeyCode::Apps,
            KeyCode::Asterisk => VirtualKeyCode::Asterisk,
            KeyCode::At => VirtualKeyCode::At,
            KeyCode::Ax => VirtualKeyCode::Ax,
            KeyCode::Backslash => VirtualKeyCode::Backslash,
            KeyCode::Calculator => VirtualKeyCode::Calculator,
            KeyCode::Capital => VirtualKeyCode::Capital,
            KeyCode::Colon => VirtualKeyCode::Colon,
            KeyCode::Comma => VirtualKeyCode::Comma,
            KeyCode::Convert => VirtualKeyCode::Convert,
            KeyCode::Equals => VirtualKeyCode::Equals,
            KeyCode::Grave => VirtualKeyCode::Grave,
            KeyCode::Kana => VirtualKeyCode::Kana,
            KeyCode::Kanji => VirtualKeyCode::Kanji,
            KeyCode::LAlt => VirtualKeyCode::LAlt,
            KeyCode::LBracket => VirtualKeyCode::LBracket,
            KeyCode::LControl => VirtualKeyCode::LControl,
            KeyCode::LShift => VirtualKeyCode::LShift,
            KeyCode::LWin => VirtualKeyCode::LWin,
            KeyCode::Mail => VirtualKeyCode::Mail,
            KeyCode::MediaSelect => VirtualKeyCode::MediaSelect,
            KeyCode::MediaStop => VirtualKeyCode::MediaStop,
            KeyCode::Minus => VirtualKeyCode::Minus,
            KeyCode::Mute => VirtualKeyCode::Mute,
            KeyCode::MyComputer => VirtualKeyCode::MyComputer,
            KeyCode::NavigateForward => VirtualKeyCode::NavigateForward,
            KeyCode::NavigateBackward => VirtualKeyCode::NavigateBackward,
            KeyCode::NextTrack => VirtualKeyCode::NextTrack,
            KeyCode::NoConvert => VirtualKeyCode::NoConvert,
            KeyCode::OEM102 => VirtualKeyCode::OEM102,
            KeyCode::Period => VirtualKeyCode::Period,
            KeyCode::PlayPause => VirtualKeyCode::PlayPause,
            KeyCode::Plus => VirtualKeyCode::Plus,
            KeyCode::Power => VirtualKeyCode::Power,
            KeyCode::PrevTrack => VirtualKeyCode::PrevTrack,
            KeyCode::RAlt => VirtualKeyCode::RAlt,
            KeyCode::RBracket => VirtualKeyCode::RBracket,
            KeyCode::RControl => VirtualKeyCode::RControl,
            KeyCode::RShift => VirtualKeyCode::RShift,
            KeyCode::RWin => VirtualKeyCode::RWin,
            KeyCode::Semicolon => VirtualKeyCode::Semicolon,
            KeyCode::Slash => VirtualKeyCode::Slash,
            KeyCode::Sleep => VirtualKeyCode::Sleep,
            KeyCode::Stop => VirtualKeyCode::Stop,
            KeyCode::Sysrq => VirtualKeyCode::Sysrq,
            KeyCode::Tab => VirtualKeyCode::Tab,
            KeyCode::Underline => VirtualKeyCode::Underline,
            KeyCode::Unlabeled => VirtualKeyCode::Unlabeled,
            KeyCode::VolumeDown => VirtualKeyCode::VolumeDown,
            KeyCode::VolumeUp => VirtualKeyCode::VolumeUp,
            KeyCode::Wake => VirtualKeyCode::Wake,
            KeyCode::WebBack => VirtualKeyCode::WebBack,
            KeyCode::WebFavorites => VirtualKeyCode::WebFavorites,
            KeyCode::WebForward => VirtualKeyCode::WebForward,
            KeyCode::WebHome => VirtualKeyCode::WebHome,
            KeyCode::WebRefresh => VirtualKeyCode::WebRefresh,
            KeyCode::WebSearch => VirtualKeyCode::WebSearch,
            KeyCode::WebStop => VirtualKeyCode::WebStop,
            KeyCode::Yen => VirtualKeyCode::Yen,
            KeyCode::Copy => VirtualKeyCode::Copy,
            KeyCode::Paste => VirtualKeyCode::Paste,
            KeyCode::Cut => VirtualKeyCode::Cut,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    Number(u32),
}

impl Into<usize> for MouseButton {
    fn into(self) -> usize {
        match self {
            MouseButton::Left => 0,
            MouseButton::Middle => 2,
            MouseButton::Right => 1,
            MouseButton::Number(button) => button as usize,
        }
    }
}
