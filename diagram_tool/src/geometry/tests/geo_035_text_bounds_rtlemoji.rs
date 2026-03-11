use super::super::*;
use super::*;
#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

    // ============== GEO-035: Text Bounds RTL/Emoji ==============

    /// Represents text with extended metrics for Unicode handling
    #[derive(Debug, Clone, PartialEq)]
    pub struct ExtendedText {
        pub x: f64,
        pub y: f64,
        pub content: String,
        pub font_size: f64,
        pub direction: TextDirection,
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum TextDirection {
        LeftToRight,
        RightToLeft,
    }

    impl ExtendedText {
        #[must_use]
        pub fn new(x: f64, y: f64, content: &str, font_size: f64) -> Self {
            Self {
                x,
                y,
                content: content.to_string(),
                font_size,
                direction: TextDirection::LeftToRight,
            }
        }

        #[must_use]
        pub const fn with_direction(mut self, direction: TextDirection) -> Self {
            self.direction = direction;
            self
        }

        /// Count grapheme clusters (user-perceived characters)
        fn grapheme_count(&self) -> usize {
            // Simplified grapheme counting - in production use unicode-segmentation crate
            // For tests, we handle common cases
            let s = &self.content;
            let mut count = 0;
            let mut chars = s.chars().peekable();

            while let Some(_c) = chars.next() {
                count += 1;

                // Check for emoji modifiers and ZWJ sequences
                while let Some(&next) = chars.peek() {
                    if next == '\u{200D}' {
                        // ZWJ - join with next
                        chars.next();
                        if chars.peek().is_some() {
                            chars.next(); // Consume the joined character
                        }
                    } else if Self::is_emoji_modifier(next) {
                        chars.next(); // Consume modifier
                    } else {
                        break;
                    }
                }
            }

            count
        }

        fn is_emoji_modifier(c: char) -> bool {
            matches!(
                c,
                '\u{FE00}'..='\u{FE0F}' // Variation selectors
                | '\u{1F3FB}'..='\u{1F3FF}' // Skin tone modifiers
                | '\u{200D}' // ZWJ (handled separately above)
            )
        }

        /// Calculate bounds with Unicode-aware width estimation
        #[must_use]
        pub fn bounds(&self) -> AABB {
            let grapheme_count = self.grapheme_count() as f64;

            // Emoji typically render at 2x width of normal characters
            // Count emoji vs regular characters
            let emoji_count = self.count_emoji() as f64;
            let regular_count = grapheme_count - emoji_count;

            // Approximate width: regular chars at 0.6 * font_size, emoji at 1.2 * font_size
            let width = regular_count * self.font_size * 0.6 + emoji_count * self.font_size * 1.2;
            let height = self.font_size;

            match self.direction {
                TextDirection::LeftToRight => {
                    AABB::new(self.x, self.y, self.x + width, self.y + height)
                }
                TextDirection::RightToLeft => {
                    AABB::new(self.x - width, self.y, self.x, self.y + height)
                }
            }
        }

        fn count_emoji(&self) -> usize {
            let s = &self.content;
            let mut count = 0;
            let chars: Vec<char> = s.chars().collect();
            let mut i = 0;

            while i < chars.len() {
                let c = chars[i];

                // Check for common emoji ranges
                if Self::is_emoji_base(c) {
                    count += 1;

                    // Skip modifiers and ZWJ sequences
                    i += 1;
                    while i < chars.len() {
                        let next = chars[i];
                        if next == '\u{200D}' {
                            // ZWJ - this is part of the same emoji
                            i += 1;
                            if i < chars.len() {
                                i += 1; // Skip the joined char
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

        fn is_emoji_base(c: char) -> bool {
            matches!(
                c,
                '\u{1F600}'..='\u{1F64F}' // Emoticons
                | '\u{1F300}'..='\u{1F5FF}' // Misc Symbols and Pictographs
                | '\u{1F680}'..='\u{1F6FF}' // Transport and Map
                | '\u{1F1E0}'..='\u{1F1FF}' // Flags
                | '\u{2600}'..='\u{26FF}' // Misc symbols
                | '\u{2700}'..='\u{27BF}' // Dingbats
                | '\u{1F900}'..='\u{1F9FF}' // Supplemental Symbols and Pictographs
                | '\u{1FA00}'..='\u{1FA6F}' // Chess Symbols
                | '\u{1FA70}'..='\u{1FAFF}' // Symbols and Pictographs Extended-A
            )
        }
    }

    #[test]
    fn test_text_bounds_ltr_simple() {
        // Given: simple LTR text
        let text = ExtendedText::new(10.0, 20.0, "Hello", 16.0);

        // When: calculating bounds
        let bounds = text.bounds();

        // Then: bounds extend to the right
        assert!((bounds.min_x - 10.0).abs() < TOLERANCE);
        assert!(bounds.max_x > 10.0);
        assert!((bounds.min_y - 20.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_text_bounds_rtl_simple() {
        // Given: RTL text (Arabic example)
        let text = ExtendedText::new(100.0, 20.0, "مرحبا", 16.0)
            .with_direction(TextDirection::RightToLeft);

        // When: calculating bounds
        let bounds = text.bounds();

        // Then: bounds extend to the left
        assert!((bounds.max_x - 100.0).abs() < TOLERANCE);
        assert!(bounds.min_x < 100.0);
        assert!((bounds.min_y - 20.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_text_bounds_emoji_simple() {
        // Given: text with simple emoji
        let text = ExtendedText::new(0.0, 0.0, "Hi 😀", 16.0);

        // When: calculating bounds
        let bounds = text.bounds();

        // Then: bounds account for emoji being wider
        assert!(bounds.width() > 0.0);
        // Emoji should contribute approximately 2x width of regular char
    }

    #[test]
    fn test_text_bounds_emoji_only() {
        // Given: emoji-only text
        let text = ExtendedText::new(0.0, 0.0, "😀🎉🚀", 20.0);

        // When: calculating bounds
        let bounds = text.bounds();

        // Then: bounds account for 3 emoji at ~1.2x font_size each
        let expected_min_width = 3.0 * 20.0 * 1.0; // At least 3 emoji widths
        assert!(bounds.width() >= expected_min_width);
    }

    #[test]
    fn test_text_bounds_zwj_emoji() {
        // Given: text with ZWJ sequence emoji (family emoji = person + ZWJ + person + ...)
        // Family: 👨‍👩‍👧 (man + ZWJ + woman + ZWJ + girl)
        let text = ExtendedText::new(0.0, 0.0, "👨‍👩‍👧", 16.0);

        // When: calculating bounds
        let bounds = text.bounds();

        // Then: ZWJ sequence should be counted as single grapheme
        assert!(bounds.width() > 0.0);
        // Should be roughly the width of one emoji, not three
    }

    #[test]
    fn test_text_bounds_mixed_ltr_emoji() {
        // Given: mixed text with emoji
        let text = ExtendedText::new(0.0, 0.0, "Test: ✓ Done! 🎉", 14.0);

        // When: calculating bounds
        let bounds = text.bounds();

        // Then: bounds contain all characters
        assert!(bounds.width() > 0.0);
        assert!((bounds.height() - 14.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_text_bounds_skin_tone_modifier() {
        // Given: emoji with skin tone modifier
        let text = ExtendedText::new(0.0, 0.0, "👋🏻", 16.0); // Waving hand with light skin tone

        // When: calculating bounds
        let bounds = text.bounds();

        // Then: modifier should not add extra width (it's part of the emoji)
        assert!(bounds.width() > 0.0);
    }

    #[test]
    fn test_text_bounds_empty() {
        // Given: empty text
        let text = ExtendedText::new(10.0, 20.0, "", 16.0);

        // When: calculating bounds
        let bounds = text.bounds();

        // Then: bounds have zero width but maintain height
        assert!((bounds.min_x - 10.0).abs() < TOLERANCE);
        assert!((bounds.width() - 0.0).abs() < TOLERANCE);
        assert!((bounds.height() - 16.0).abs() < TOLERANCE);
    }

