pub use super::keyboard::Key;
pub use super::uibuf::{CharStyle, CharColor};

pub use self::rb::RustboxFrontend;

pub enum EditorEvent {
    KeyEvent(Option<Key>),
    UnSupported
}

pub trait Frontend {
    fn poll_event(&self) -> EditorEvent;
    fn present(&self);
    fn get_window_height(&self) -> uint;
    fn get_window_width(&self) -> uint;
    fn draw_cursor(&mut self, offset: int, linenum: int);
    fn draw_char(&mut self, offset: uint, linenum: uint, ch: char, fg: CharColor, bg: CharColor, style: CharStyle);
}

mod rb;
