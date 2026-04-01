#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rectangle {
    pub position: Position,
    pub size: Size,
}

impl Rectangle {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            position: Position { x, y },
            size: Size { width, height },
        }
    }

    pub fn contains_point(&self, point: Position) -> bool {
        let right = self.position.x + self.size.width as i32;
        let bottom = self.position.y + self.size.height as i32;

        point.x >= self.position.x
            && point.x < right
            && point.y >= self.position.y
            && point.y < bottom
    }

    pub fn intersects(&self, other: Rectangle) -> bool {
        let self_right = self.position.x + self.size.width as i32;
        let self_bottom = self.position.y + self.size.height as i32;
        let other_right = other.position.x + other.size.width as i32;
        let other_bottom = other.position.y + other.size.height as i32;

        self.position.x < other_right
            && self_right > other.position.x
            && self.position.y < other_bottom
            && self_bottom > other.position.y
    }

    pub fn intersection(&self, other: Rectangle) -> Option<Rectangle> {
        if !self.intersects(other) {
            return None;
        }

        let self_right = self.position.x + self.size.width as i32;
        let self_bottom = self.position.y + self.size.height as i32;
        let other_right = other.position.x + other.size.width as i32;
        let other_bottom = other.position.y + other.size.height as i32;

        let x = self.position.x.max(other.position.x);
        let y = self.position.y.max(other.position.y);
        let right = self_right.min(other_right);
        let bottom = self_bottom.min(other_bottom);

        let width = (right - x) as u32;
        let height = (bottom - y) as u32;

        Some(Rectangle::new(x, y, width, height))
    }

    pub fn offset(&self, dx: i32, dy: i32) -> Rectangle {
        Rectangle {
            position: Position {
                x: self.position.x + dx,
                y: self.position.y + dy,
            },
            size: self.size,
        }
    }
}
