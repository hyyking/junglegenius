pub mod inhibitor;
pub mod turret;

pub mod nexus;

pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

pub const MAP_BOUNDS: Rectangle = Rectangle {
    x: -120.0,
    y: -120.0,
    width: 14980.0,
    height: 14980.0,
};
