use geo::{
    Area, BoundingRect, Centroid, Contains, EuclideanDistance, EuclideanLength,
    Intersects, LineString, LinesIter, MinimumRotatedRect, MultiPolygon, Polygon, Simplify,
};
use rstar::PointDistance;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[derive(Debug, Clone)]
pub struct OOBB {
    polygon: Polygon<f32>,
}

impl PartialEq for OOBB {
    fn eq(&self, other: &Self) -> bool {
        self.polygon.exterior() == other.polygon.exterior()
    }
}

impl OOBB {
    pub fn from_corners(a: [f32; 2], b: [f32; 2]) -> Self {
        Self {
            polygon: geo::Rect::new(a, b).into(),
        }
    }

    pub fn from_polygon(poly: Polygon<f32>) -> Self {
        Self {
            polygon: poly.minimum_rotated_rect().unwrap()
        }
    }

    pub fn lines(&self) -> impl Iterator<Item = geo::Line<f32>> + '_ {
        self.polygon.lines_iter()
    }
}

impl rstar::Envelope for OOBB {
    type Point = [f32; 2];

    fn new_empty() -> Self {
        Self {
            polygon: Polygon::new(LineString::new(vec![]), vec![]),
        }
    }

    fn contains_point(&self, point: &Self::Point) -> bool {
        self.polygon
            .contains(&geo::point! { x: point[0], y: point[1] })
    }

    fn contains_envelope(&self, rhs: &Self) -> bool {
        self.polygon.contains(&rhs.polygon)
    }

    fn merge(&mut self, other: &Self) {
        let new = MultiPolygon::new(vec![self.polygon.clone(), other.polygon.clone()]);
        self.polygon = new
            .simplify(&0.1)
            .minimum_rotated_rect()
            .expect("no mrc found");
    }

    fn merged(&self, other: &Self) -> Self {
        let new = MultiPolygon::new(vec![self.polygon.clone(), other.polygon.clone()]);
        Self {
            polygon: new
                .simplify(&0.1)
                .minimum_rotated_rect()
                .expect("no mrc found"),
        }
    }

    fn intersects(&self, other: &Self) -> bool {
        self.polygon.intersects(&other.polygon)
    }

    #[rustfmt::skip]
    fn intersection_area(&self, other: &Self) -> <Self::Point as rstar::Point>::Scalar {
        if self.polygon.contains(&other.polygon) {
            return other.polygon.unsigned_area();
        }

        let samples = 4;

        let a = other
            .polygon
            .lines_iter()
            .filter(|line| line.intersects(&self.polygon))
            .flat_map(|l| {
                (0..samples).map(move |i| {
                    let x = ((i as f32) / (samples as f32)) * l.dx();
                    geo::point! {
                        x: l.start.x + x,
                        y: l.start.y + x * l.slope()
                    }
                })
            })
            .filter(|p| self.polygon.contains(p));

        let b = self
            .polygon
            .lines_iter()
            .filter(|line| line.intersects(&other.polygon))
            .flat_map(|l| {
                (0..samples).map(move |i| {
                    let x = ((i as f32) / (samples as f32)) * l.dx();
                    geo::point! {
                        x: l.start.x + x,
                        y: l.start.y + x * l.slope()
                    }
                })
            })
            .filter(|p| other.polygon.contains(p));
        let points: LineString<f32> = a.chain(b).collect();

        (!points.0.is_empty())
            .then(|| points.bounding_rect())
            .flatten()
            .as_ref()
            .map(Area::unsigned_area)
            .unwrap_or(0.0)
    }

    fn area(&self) -> <Self::Point as rstar::Point>::Scalar {
        self.polygon.unsigned_area()
    }

    fn distance_2(&self, point: &Self::Point) -> <Self::Point as rstar::Point>::Scalar {
        self.polygon
            .exterior()
            .euclidean_distance(&geo::point! {x: point[0], y: point[1]})
    }

    fn min_max_dist_2(&self, point: &Self::Point) -> <Self::Point as rstar::Point>::Scalar {
        self.polygon
            .exterior()
            .points()
            .map(|p| p.distance_2(&geo::point! {x: point[0], y: point[1]}))
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .expect("no points")
    }

    fn center(&self) -> Self::Point {
        let p = self.polygon.centroid().expect("no centroid");
        [p.x(), p.y()]
    }

    fn perimeter_value(&self) -> <Self::Point as rstar::Point>::Scalar {
        self.polygon
            .exterior()
            .lines()
            .map(|l| l.euclidean_length())
            .sum()
    }

    fn sort_envelopes<T: rstar::RTreeObject<Envelope = Self>>(axis: usize, envelopes: &mut [T]) {
        envelopes.sort_by(|a, b| {
            let bba = a
                .envelope()
                .polygon
                .exterior()
                .bounding_rect()
                .unwrap()
                .min();
            let bbb = b
                .envelope()
                .polygon
                .exterior()
                .bounding_rect()
                .unwrap()
                .min();

            match axis {
                0 => bba.x.partial_cmp(&bbb.x).unwrap(),
                1 => bba.y.partial_cmp(&bbb.y).unwrap(),
                _ => panic!("dimension 2 only"),
            }
        });
    }

    fn partition_envelopes<T: rstar::RTreeObject<Envelope = Self>>(
        axis: usize,
        envelopes: &mut [T],
        selection_size: usize,
    ) {
        envelopes.select_nth_unstable_by(selection_size, |a, b| {
            let bba = a
                .envelope()
                .polygon
                .exterior()
                .bounding_rect()
                .unwrap()
                .min();
            let bbb = b
                .envelope()
                .polygon
                .exterior()
                .bounding_rect()
                .unwrap()
                .min();

            match axis {
                0 => bba.x.partial_cmp(&bbb.x).unwrap(),
                1 => bba.y.partial_cmp(&bbb.y).unwrap(),
                _ => panic!("dimension 2 only"),
            }
        });
    }
}
