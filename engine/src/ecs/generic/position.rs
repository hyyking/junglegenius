

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PositionComponent {
    pub point: lyon::math::Point,
    pub radius: f32,
}

impl rstar::RTreeObject for PositionComponent {
    type Envelope = oobb::OOBB;

    fn envelope(&self) -> Self::Envelope {
        let [x, y] = self.point.to_array();

        oobb::OOBB::from_corners(
            [x - self.radius, y - self.radius],
            [x + self.radius, y + self.radius],
        )
    }
}

impl rstar::PointDistance for PositionComponent {
    fn distance_2(&self, point: &[f32; 2]) -> f32 {
        let origin = self.point.to_array();
        let d_x = origin[0] - point[0];
        let d_y = origin[1] - point[1];
        let distance_to_origin = (d_x * d_x + d_y * d_y).sqrt();
        let distance_to_ring = distance_to_origin - self.radius;
        let distance_to_circle = f32::max(0.0, distance_to_ring);
        // We must return the squared distance!
        distance_to_circle * distance_to_circle
    }

    // This implementation is not required but more efficient since it
    // omits the calculation of a square root
    fn contains_point(&self, point: &[f32; 2]) -> bool {
        let origin = self.point.to_array();
        let d_x = origin[0] - point[0];
        let d_y = origin[1] - point[1];
        let distance_to_origin_2 = d_x * d_x + d_y * d_y;
        let radius_2 = self.radius * self.radius;
        distance_to_origin_2 <= radius_2
    }
}
