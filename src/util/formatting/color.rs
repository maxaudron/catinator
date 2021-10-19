pub enum Color {
    White = 00,
    Black = 01,
    Blue = 02,
    Green = 03,
    Red = 04,
    Brown = 05,
    Magenta = 06,
    Orange = 07,
    Yellow = 08,
    LightGreen = 09,
    Cyan = 10,
    LightCyan = 11,
    LightBlue = 12,
    Pink = 13,
    Grey = 14,
    LightGrey = 15,
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Color::White => f.write_str("\x00"),
            Color::Black => f.write_str("\x01"),
            Color::Blue => f.write_str("\x02"),
            Color::Green => f.write_str("\x03"),
            Color::Red => f.write_str("\x04"),
            Color::Brown => f.write_str("\x05"),
            Color::Magenta => f.write_str("\x06"),
            Color::Orange => f.write_str("\x07"),
            Color::Yellow => f.write_str("\x08"),
            Color::LightGreen => f.write_str("\x09"),
            Color::Cyan => f.write_str("\x10"),
            Color::LightCyan => f.write_str("\x11"),
            Color::LightBlue => f.write_str("\x12"),
            Color::Pink => f.write_str("\x13"),
            Color::Grey => f.write_str("\x14"),
            Color::LightGrey => f.write_str("\x15"),
        }
    }
}
