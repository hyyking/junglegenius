use geo::{Contains, EuclideanDistance};

use crate::ecs::{generic::PositionComponent, UnitId};

#[derive(Debug, PartialEq)]
pub enum CollisionBox {
    Polygon(geo::Polygon<f32>),
    Unit {
        position: PositionComponent,
        guid: UnitId,
    },
}

impl rstar::RTreeObject for CollisionBox {
    type Envelope = oobb::OOBB<f32>;

    fn envelope(&self) -> Self::Envelope {
        match self {
            CollisionBox::Polygon(poly) => oobb::OOBB::from_polygon(poly.clone()),
            CollisionBox::Unit { position, .. } => position.envelope(),
        }
    }
}

pub struct NavigationMap {
    pub tree: rstar::RTree<CollisionBox>,
}

impl rstar::PointDistance for CollisionBox {
    fn distance_2(&self, point: &[f32; 2]) -> f32 {
        match self {
            CollisionBox::Polygon(poly) => poly
                .euclidean_distance(&geo::point! {x: point[0], y: point[1]})
                .exp2(),
            CollisionBox::Unit { position, .. } => {
                let origin = position.point.to_array();
                let d_x = origin[0] - point[0];
                let d_y = origin[1] - point[1];
                let distance_to_origin = (d_x * d_x + d_y * d_y).sqrt();
                let distance_to_ring = distance_to_origin - position.radius;
                let distance_to_circle = f32::max(0.0, distance_to_ring);
                // We must return the squared distance!
                distance_to_circle * distance_to_circle
            }
        }
    }

    // This implementation is not required but more efficient since it
    // omits the calculation of a square root
    fn contains_point(&self, point: &[f32; 2]) -> bool {
        match self {
            CollisionBox::Polygon(poly) => poly.contains(&geo::point! {x: point[0], y: point[1]}),
            CollisionBox::Unit { position, .. } => {
                let origin = position.point.to_array();
                let d_x = origin[0] - point[0];
                let d_y = origin[1] - point[1];
                let distance_to_origin_2 = d_x * d_x + d_y * d_y;
                let radius_2 = position.radius * position.radius;
                distance_to_origin_2 <= radius_2
            }
        }
    }
}
