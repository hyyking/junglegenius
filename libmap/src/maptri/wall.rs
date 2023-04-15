use geo::EuclideanDistance;
use rstar::PointDistance;


pub struct Wall(pub geo::Polygon, pub String);

impl PointDistance for Wall {
    fn distance_2(
        &self,
        point: &<Self::Envelope as rstar::Envelope>::Point,
    ) -> <<Self::Envelope as rstar::Envelope>::Point as rstar::Point>::Scalar {
        self.0
            .euclidean_distance(&geo::point! {x: point[0], y: point[1]})
    }
}
impl rstar::RTreeObject for Wall {
    type Envelope = oobb::OOBB<f64>;

    fn envelope(&self) -> Self::Envelope {
        oobb::OOBB::from_polygon(self.0.clone())
    }
}
