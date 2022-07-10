use glam::Vec2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pos: Vec2,
    size: Vec2,
}

impl Rect {
    pub const ZERO: Self = Self {
        pos: Vec2::ZERO,
        size: Vec2::ZERO,
    };

    pub const ONE: Self = Self {
        pos: Vec2::ZERO,
        size: Vec2::ONE,
    };

    #[inline]
    pub fn from_pos_size(pos: Vec2, size: Vec2) -> Self {
        Self { pos, size }
    }

    #[inline]
    pub fn pos(&self) -> Vec2 {
        self.pos
    }

    #[inline]
    pub fn size(&self) -> Vec2 {
        self.size
    }

    #[inline]
    pub fn max(&self) -> Vec2 {
        self.pos + self.size
    }

    #[inline]
    pub fn set_pos(&mut self, pos: Vec2) {
        self.pos = pos;
    }

    #[inline]
    pub fn set_size(&mut self, size: Vec2) {
        self.size = size;
    }

    #[inline]
    pub fn contains_point(&self, point: Vec2) -> bool {
        point.x >= self.pos.x
            && point.x <= self.pos.x + self.size.x
            && point.y >= self.pos.y
            && point.y <= self.pos.y + self.size.y
    }

    #[inline]
    pub fn intersects(&self, other: &Self) -> bool {
        let self_max = self.max();
        let other_max = other.max();

        let x_intersect = self.pos.x < other_max.x && self_max.x > other.pos.x;
        let y_intersect = self.pos.y < other_max.y && self_max.y > other.pos.y;

        x_intersect && y_intersect
    }

    #[inline]
    pub fn div_vec2(&self, size: Vec2) -> Self {
        Self::from_pos_size(self.pos / size, self.size / size)
    }
}