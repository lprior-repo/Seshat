use super::aabb::AABB;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TextContent(pub String);

/// Represents text with position and font metrics
#[derive(Debug, Clone, PartialEq)]
pub struct Text {
    pub x: f64,
    pub y: f64,
    pub content: TextContent,
    pub font_size: f64,
}

impl Text {
    #[must_use]
    pub fn new(x: f64, y: f64, content: &str, font_size: f64) -> Self {
        Self {
            x,
            y,
            content: TextContent(content.to_string()),
            font_size,
        }
    }

    #[must_use]
    pub fn bounds(&self) -> AABB {
        #[allow(clippy::cast_precision_loss)]
        let char_count = self.content.0.chars().count() as f64;
        let width = self.font_size * 0.6 * char_count;
        let height = self.font_size;
        AABB::new(self.x, self.y, self.x + width, self.y + height)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextDirection {
    LeftToRight,
    RightToLeft,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExtendedText {
    pub x: f64,
    pub y: f64,
    pub content: TextContent,
    pub font_size: f64,
    pub direction: TextDirection,
}

impl ExtendedText {
    #[must_use]
    pub fn new(x: f64, y: f64, content: &str, font_size: f64) -> Self {
        Self {
            x,
            y,
            content: TextContent(content.to_string()),
            font_size,
            direction: TextDirection::LeftToRight,
        }
    }

    #[must_use]
    pub const fn with_direction(mut self, direction: TextDirection) -> Self {
        self.direction = direction;
        self
    }

    fn grapheme_count(&self) -> usize {
        let s = &self.content.0;
        let mut count = 0;
        let mut chars = s.chars().peekable();

        while let Some(_c) = chars.next() {
            count += 1;
            while let Some(&next) = chars.peek() {
                if next == '\u{200D}' {
                    chars.next();
                    if chars.peek().is_some() {
                        chars.next();
                    }
                } else if Self::is_emoji_modifier(next) {
                    chars.next();
                } else {
                    break;
                }
            }
        }
        count
    }

    const fn is_emoji_modifier(c: char) -> bool {
        matches!(
            c,
            '\u{FE00}'..='\u{FE0F}' | '\u{1F3FB}'..='\u{1F3FF}' | '\u{200D}'
        )
    }

    #[must_use]
    pub fn bounds(&self) -> AABB {
        #[allow(clippy::cast_precision_loss)]
        let emoji_count = self.count_emoji() as f64;
        #[allow(clippy::cast_precision_loss)]
        let regular_count = self.grapheme_count() as f64 - emoji_count;
        let width =
            (regular_count * self.font_size).mul_add(0.6, emoji_count * self.font_size * 1.2);
        match self.direction {
            TextDirection::LeftToRight => {
                AABB::new(self.x, self.y, self.x + width, self.y + self.font_size)
            }
            TextDirection::RightToLeft => {
                AABB::new(self.x - width, self.y, self.x, self.y + self.font_size)
            }
        }
    }

    fn count_emoji(&self) -> usize {
        let s = &self.content.0;
        let mut count = 0;
        let chars: Vec<char> = s.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let c = chars[i];
            if Self::is_emoji_base(c) {
                count += 1;
                i += 1;
                while i < chars.len() {
                    let next = chars[i];
                    if next == '\u{200D}' {
                        i += 1;
                        if i < chars.len() {
                            i += 1;
                        }
                    } else if Self::is_emoji_modifier(next) {
                        i += 1;
                    } else {
                        break;
                    }
                }
            } else {
                i += 1;
            }
        }
        count
    }

    const fn is_emoji_base(c: char) -> bool {
        matches!(
            c,
            '\u{1F600}'..='\u{1F64F}' | '\u{1F300}'..='\u{1F5FF}' | '\u{1F680}'..='\u{1F6FF}' | '\u{1F1E0}'..='\u{1F1FF}' | '\u{2600}'..='\u{26FF}' | '\u{2700}'..='\u{27BF}' | '\u{1F900}'..='\u{1F9FF}' | '\u{1FA00}'..='\u{1FA6F}' | '\u{1FA70}'..='\u{1FAFF}'
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_new() {
        let text = Text::new(10.0, 20.0, "hello", 16.0);
        assert_eq!(text.x, 10.0);
        assert_eq!(text.y, 20.0);
        assert_eq!(text.content.0, "hello");
        assert_eq!(text.font_size, 16.0);
    }

    #[test]
    fn test_text_bounds() {
        let text = Text::new(10.0, 20.0, "hello", 10.0);
        let bounds = text.bounds();
        // 5 chars * 10.0 * 0.6 = 30.0 width
        assert_eq!(bounds.min_x, 10.0);
        assert_eq!(bounds.min_y, 20.0);
        assert_eq!(bounds.max_x, 40.0); // 10.0 + 30.0
        assert_eq!(bounds.max_y, 30.0); // 20.0 + 10.0
    }

    #[test]
    fn test_extended_text_new_and_with_direction() {
        let t1 = ExtendedText::new(10.0, 20.0, "hello", 16.0);
        assert_eq!(t1.direction, TextDirection::LeftToRight);
        let t2 = t1.with_direction(TextDirection::RightToLeft);
        assert_eq!(t2.direction, TextDirection::RightToLeft);
    }

    #[test]
    fn test_extended_text_bounds_ltr() {
        let t1 = ExtendedText::new(10.0, 20.0, "hello", 10.0);
        let bounds = t1.bounds();
        assert_eq!(bounds.min_x, 10.0);
        assert_eq!(bounds.max_x, 40.0);
    }

    #[test]
    fn test_extended_text_bounds_rtl() {
        let t1 =
            ExtendedText::new(10.0, 20.0, "hello", 10.0).with_direction(TextDirection::RightToLeft);
        let bounds = t1.bounds();
        assert_eq!(bounds.min_x, -20.0); // 10.0 - 30.0
        assert_eq!(bounds.max_x, 10.0);
    }

    #[test]
    fn test_extended_text_emoji_counting() {
        let t1 = ExtendedText::new(0.0, 0.0, "he😃llo", 10.0);
        let bounds = t1.bounds();
        // 5 regular chars = 5 * 10 * 0.6 = 30
        // 1 emoji = 1 * 10 * 1.2 = 12
        // total width = 42
        assert_eq!(bounds.max_x, 42.0);
    }
}
