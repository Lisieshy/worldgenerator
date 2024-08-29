#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockProperties {
    pub collidable: bool,
    pub opaque: bool,
    pub is_air: bool,
}

impl Default for BlockProperties {
    fn default() -> Self {
        Self {
            collidable: true,
            opaque: true,
            is_air: false,
        }
    }
}