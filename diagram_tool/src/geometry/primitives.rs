/// Represents a 2D point
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    #[must_use]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub const fn origin() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

/// Represents an axis-aligned bounding box
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct AABB {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl AABB {
    #[must_use]
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        Self {
            min_x: min_x.min(max_x),
            min_y: min_y.min(max_y),
            max_x: min_x.max(max_x),
            max_y: min_y.max(max_y),
        }
    }

    #[must_use]
    pub fn width(&self) -> f64 {
        self.max_x - self.min_x
    }

    #[must_use]
    pub fn height(&self) -> f64 {
        self.max_y - self.min_y
    }

    #[must_use]
    pub fn center(&self) -> Point {
        Point::new(
            self.min_x + self.width() / 2.0,
            self.min_y + self.height() / 2.0,
        )
    }

    /// Expand the AABB by a given amount on all sides
    #[must_use]
    pub fn expand(&self, amount: f64) -> Self {
        Self::new(
            self.min_x - amount,
            self.min_y - amount,
            self.max_x + amount,
            self.max_y + amount,
        )
    }

    #[must_use]
    pub fn contains_point(&self, point: &Point) -> bool {
        point.x >= self.min_x && point.x <= self.max_x
            && point.y >= self.min_y && point.y <= self.max_y
    }

    /// Compute the union of two AABBs
    #[must_use]
    pub fn union(&self, other: &Self) -> Self {
        Self::new(
            self.min_x.min(other.min_x),
            self.min_y.min(other.min_y),
            self.max_x.max(other.max_x),
            self.max_y.max(other.max_y),
        )
    }
}

/// Represents a rectangle with position, dimensions, and optional rotation
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rectangle {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub rotation: f64, // rotation in radians
}

impl Rectangle {
    #[must_use]
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
            rotation: 0.0,
        }
    }

    #[must_use]
    pub const fn with_rotation(mut self, rotation: f64) -> Self {
        self.rotation = rotation;
        self
    }

    #[must_use]
    pub fn aabb(&self) -> AABB {
        if self.rotation == 0.0 {
            AABB::new(self.x, self.y, self.x + self.width, self.y + self.height)
        } else {
            let corners = self.corners();
            let mut min_x = f64::INFINITY;
            let mut min_y = f64::INFINITY;
            let mut max_x = f64::NEG_INFINITY;
            let mut max_y = f64::NEG_INFINITY;

            for corner in corners {
                min_x = min_x.min(corner.x);
                min_y = min_y.min(corner.y);
                max_x = max_x.max(corner.x);
                max_y = max_y.max(corner.y);
            }

            AABB::new(min_x, min_y, max_x, max_y)
        }
    }

    #[must_use]
    pub fn corners(&self) -> [Point; 4] {
        let cx = self.x + self.width / 2.0;
        let cy = self.y + self.height / 2.0;

        let hw = self.width / 2.0;
        let hh = self.height / 2.0;

        let local_corners = [
            Point::new(-hw, -hh),
            Point::new(hw, -hh),
            Point::new(hw, hh),
            Point::new(-hw, hh),
        ];

        let cos = self.rotation.cos();
        let sin = self.rotation.sin();

        local_corners.map(|p| {
            Point::new(
                p.x.mul_add(cos, -(p.y * sin)) + cx,
                p.x.mul_add(sin, p.y * cos) + cy,
            )
        })
    }
}

/// Represents a shape with stroke
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct StrokedShape<T> {
    pub shape: T,
    pub stroke_width: f64,
}

impl<T> StrokedShape<T> {
    #[must_use]
    pub const fn new(shape: T, stroke_width: f64) -> Self {
        Self {
            shape,
            stroke_width,
        }
    }
}

impl StrokedShape<Rectangle> {
    #[must_use]
    pub fn bounds_with_stroke(&self) -> AABB {
        let shape_aabb = self.shape.aabb();
        shape_aabb.expand(self.stroke_width / 2.0)
    }
}

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
        let emoji_count = self.count_emoji() as f64;
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

/// Represents an image with position and dimensions
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Image {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Image {
    #[must_use]
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    #[must_use]
    pub fn bounds(&self) -> AABB {
        AABB::new(self.x, self.y, self.x + self.width, self.y + self.height)
    }
}
