//! Tools for formatting irc messages

mod truncate;
mod color;

pub use truncate::*;
pub use color::*;

/// Turn strings bold, italic,underline, strikethrough, and monospace.
///
/// The Formatting trait is implemented on `String` and `&str` for your convenience
///
/// # Format a string
/// ```
/// use catinator::util::Formatting;
///
/// let text = "this should be bold";
/// assert_eq!("\x02this should be bold\x02", text.bold())
/// ```
///
/// # Use raw formatting code
/// ```
/// use catinator::util::Formatting;
///
/// let text = "this should be bold";
/// let result = format!("{}{}", String::BOLD, text);
/// assert_eq!("\x02this should be bold", result)
/// ```
pub trait Formatting<'r> {
    const BOLD: &'r str = "\x02";
    const ITALIC: &'r str = "\x1D";
    const UNDERLINE: &'r str = "\x1F";
    const STRIKETHROUGH: &'r str = "\x1E";
    const MONOSPACE: &'r str = "\x11";
    const COLOR: &'r str = "\x03";

    fn bold(self) -> String;
    fn italic(self) -> String;
    fn underline(self) -> String;
    fn strikethrough(self) -> String;
    fn monospace(self) -> String;
    // fn color(self, color: Color) -> Self;
}

impl<'r> Formatting<'r> for String {
    fn bold(mut self) -> Self {
        self.insert_str(0, Self::BOLD);
        self.push_str(Self::BOLD);

        return self;
    }

    fn italic(mut self) -> Self {
        self.insert_str(0, Self::ITALIC);
        self.push_str(Self::ITALIC);

        return self;
    }

    fn underline(mut self) -> Self {
        self.insert_str(0, Self::UNDERLINE);
        self.push_str(Self::UNDERLINE);

        return self;
    }

    fn strikethrough(mut self) -> Self {
        self.insert_str(0, Self::STRIKETHROUGH);
        self.push_str(Self::STRIKETHROUGH);

        return self;
    }

    fn monospace(mut self) -> Self {
        self.insert_str(0, Self::MONOSPACE);
        self.push_str(Self::MONOSPACE);

        return self;
    }

    // TODO implement color codes
    // fn color(mut self, foreground: Option<Color>, background: Option<Color>) -> Self {
    //     self.insert_str(0, Self::COLOR);
    //     self.push_str(Self::COLOR);

    //     return self;
    // }
}

impl<'r> Formatting<'r> for &str {
    fn bold(self) -> String {
        format!("{}{}{}", Self::BOLD, self, Self::BOLD)
    }

    fn italic(self) -> String {
        format!("{}{}{}", Self::ITALIC, self, Self::ITALIC)
    }

    fn underline(self) -> String {
        format!("{}{}{}", Self::UNDERLINE, self, Self::UNDERLINE)
    }

    fn strikethrough(self) -> String {
        format!("{}{}{}", Self::STRIKETHROUGH, self, Self::STRIKETHROUGH)
    }

    fn monospace(self) -> String {
        format!("{}{}{}", Self::MONOSPACE, self, Self::MONOSPACE)
    }

    // TODO implement color codes
    // fn color(mut self, foreground: Option<Color>, background: Option<Color>) -> Self { }
}

#[cfg(all(test, feature = "bench"))]
mod bench {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_mut(b: &mut Bencher) {
        b.iter(|| {
            let mut src = "test".to_string();
            src.bold()
        });
    }

    #[bench]
    fn bench_format(b: &mut Bencher) {
        b.iter(|| {
            let mut src = "test".to_string();
            src.italic()
        });
    }
}
