pub mod cvt;
pub mod refined;
pub mod wall;

use std::{collections::HashSet, io::Write};

use geo::{
    BoundingRect, ConcaveHull, CoordsIter, Extremes, LineString, Point, Polygon,
    RemoveRepeatedPoints,
};
use geojson::Geometry;
use rstar::{primitives::GeomWithData, PointDistance};
use spade::{ConstrainedDelaunayTriangulation, Point2, Triangulation};
use {cvt::poly_from_voronoi_face, wall::Wall};

use rayon::prelude::*;

use crate::pipe::Pipe;

use self::refined::RefinedTesselation;

struct NavMapTriangulation {
    walls: rstar::RTree<Wall>,
    nav: rstar::RTree<[f64; 2]>,
}

impl NavMapTriangulation {
    fn populate_walls(&mut self, wall: &Polygon, id: String) -> isize {
        const CLOSE: f64 = 4.0 * 4.0;

        let mut ext = wall
            .exterior_coords_iter()
            .map(|coord| {
                match self
                    .nav
                    .locate_within_distance(
                        [coord.x.max(0.0).min(14980.0), coord.y.max(0.0).min(14980.0)],
                        CLOSE,
                    )
                    .min_by(|[ax, ay], [bx, by]| {
                        let a =
                            geo::coord! {x: ax.max(0.0).min(14980.0), y: ay.max(0.0).min(14980.0)}
                                .distance_2(&coord);
                        let b =
                            geo::coord! {x: bx.max(0.0).min(14980.0), y: by.max(0.0).min(14980.0)}
                                .distance_2(&coord);
                        a.total_cmp(&b)
                    }) {
                    Some(p) => Point::new(p[0], p[1]),
                    None => {
                        self.nav
                            .insert([coord.x.max(0.0).min(14980.0), coord.y.max(0.0).min(14980.0)]);
                        Point::new(coord.x.max(0.0).min(14980.0), coord.y.max(0.0).min(14980.0))
                    }
                }
            })
            .collect::<LineString>()
            .remove_repeated_points();

        ext.close();

        let int = wall
            .interiors()
            .iter()
            .map(|interior| {
                let mut interior = interior
                    .coords()
                    .map(|coord| {
                        match self
                            .nav
                            .locate_within_distance([coord.x, coord.y], CLOSE)
                            .min_by(|[ax, ay], [bx, by]| {
                                let a = geo::coord! {x: *ax, y: *ay}.distance_2(&coord);
                                let b = geo::coord! {x: *bx, y: *by}.distance_2(&coord);
                                a.total_cmp(&b)
                            }) {
                            Some(p) => Point::new(p[0], p[1]),
                            None => {
                                self.nav.insert([coord.x, coord.y]);
                                Point::new(coord.x, coord.y)
                            }
                        }
                    })
                    .collect::<LineString>()
                    .remove_repeated_points();
                interior.close();
                interior
            })
            .collect::<Vec<_>>();

        let poly = Wall(Polygon::new(ext, int), id);
        let new = wall.coords_count() as isize - poly.0.coords_count() as isize;
        self.walls.insert(poly);
        new
    }

    fn build(&self) -> spade::ConstrainedDelaunayTriangulation<Point2<f64>> {
        let mut cdt = spade::ConstrainedDelaunayTriangulation::<Point2<f64>>::new();
        let mut points = rstar::RTree::new();

        for &[x, y] in self.nav.iter() {
            if x < 10.0 || y < 10.0 || x > 14930.0 || y > 14930.0 {
                continue;
            }
            let p = spade::mitigate_underflow(Point2::new(x, y));
            points.insert(GeomWithData::new([p.x, p.y], cdt.insert(p).unwrap()));
        }

        if let Some(Wall(top_left, _)) = self.walls.locate_at_point(&[1.0, 1.0]) {
            let rect = top_left.extremes().unwrap();
            let p1 = spade::mitigate_underflow(Point2::new(rect.y_max.coord.x, rect.y_max.coord.y));
            let p2 = spade::mitigate_underflow(Point2::new(rect.x_max.coord.x, rect.x_max.coord.y));

            info!(y_max=?p1, x_max=?p2, "found top_left wall");

            let v1 = cdt.insert(p1).unwrap();
            points.insert(GeomWithData::new([p1.x, p1.y], v1));
            let v2 = cdt.insert(p2).unwrap();
            points.insert(GeomWithData::new([p2.x, p2.y], v2));
        } else {
            let bound = Point2::new(0.0, 0.0);
            info!(?bound, "inserting top_left bound");
            let top_left = cdt.insert(bound).unwrap();
            points.insert(GeomWithData::new([bound.x, bound.y], top_left));
        }

        if let Some(Wall(top_right, _)) = self.walls.locate_at_point(&[14979.0, 1.0]) {
            let rect = top_right.extremes().unwrap();
            let p1 = spade::mitigate_underflow(Point2::new(rect.y_max.coord.x, rect.y_max.coord.y));
            let p2 = spade::mitigate_underflow(Point2::new(rect.x_min.coord.x, rect.x_min.coord.y));

            info!(y_max=?p1, x_min=?p2, "found top_right wall");

            let v1 = cdt.insert(p1).unwrap();
            points.insert(GeomWithData::new([p1.x, p1.y], v1));
            let v2 = cdt.insert(p2).unwrap();
            points.insert(GeomWithData::new([p2.x, p2.y], v2));
        } else {
            let bound = Point2::new(14980.0, 0.0);
            info!(?bound, "inserting top_right bound");
            let top_right = cdt.insert(bound).unwrap();
            points.insert(GeomWithData::new([bound.x, bound.y], top_right));
        }

        if let Some(Wall(bot_right, _)) = self.walls.locate_at_point(&[14970.0, 14970.0]) {
            let rect = bot_right.extremes().unwrap();
            let p1 = spade::mitigate_underflow(Point2::new(rect.y_min.coord.x, rect.y_min.coord.y));
            let p2 = spade::mitigate_underflow(Point2::new(rect.x_min.coord.x, rect.x_min.coord.y));
            info!(?rect, "found bot_right wall");

            info!(y_min=?p1, x_min=?p2, "found bot_right wall");

            let v1 = cdt.insert(p1).unwrap();
            points.insert(GeomWithData::new([p1.x, p1.y], v1));
            let v2 = cdt.insert(p2).unwrap();
            points.insert(GeomWithData::new([p2.x, p2.y], v2));
        } else {
            let bound = Point2::new(14980.0, 14980.0);
            info!(?bound, "inserting bot_right bound");
            let bot_right = cdt.insert(bound).unwrap();
            points.insert(GeomWithData::new([bound.x, bound.y], bot_right));
        }

        if let Some(Wall(bot_left, _)) = self.walls.locate_at_point(&[1.0, 14979.0]) {
            let rect = bot_left.extremes().unwrap();
            let p1 = spade::mitigate_underflow(Point2::new(rect.y_min.coord.x, rect.y_min.coord.y));
            let p2 = spade::mitigate_underflow(Point2::new(rect.x_max.coord.x, rect.x_max.coord.y));

            info!(y_max=?p1, x_min=?p2, "found bot_left wall");

            let v1 = cdt.insert(p1).unwrap();
            points.insert(GeomWithData::new([p1.x, p1.y], v1));
            let v2 = cdt.insert(p2).unwrap();
            points.insert(GeomWithData::new([p2.x, p2.y], v2));
        } else {
            let bound = Point2::new(0.0, 14980.0);
            info!(?bound, "inserting bot_left bound");
            let bot_left = cdt.insert(bound).unwrap();
            points.insert(GeomWithData::new([bound.x, bound.y], bot_left));
        }


        let hull = points
            .into_iter()
            .map(|p| geo::point! {x: p.geom()[0], y: p.geom()[1]})
            .collect::<geo::MultiPoint>()
            .concave_hull(0.01);

        let f = std::fs::File::create("hull.json").unwrap();
        geojson::ser::to_feature_writer(
            f,
            &geojson::Feature {
                bbox: None,
                geometry: Some((&hull).into()),
                id: None,
                properties: None,
                foreign_members: None,
            },
        )
        .unwrap();
        cdt.add_constraint_edges(
            hull.coords_iter().map(|coord| {
                spade::mitigate_underflow(Point2::new(
                    coord.x.max(0.0).min(14980.0),
                    coord.y.max(0.0).min(14980.0),
                ))
            }),
            true,
        )
        .unwrap();

        self.walls
            .iter()
            .filter(|Wall(_, id)| id != "bot_bound" && id != "top_bound")
            .map(|Wall(wall, _)| {
                wall.coords_iter().map(|coord| {
                    spade::mitigate_underflow(Point2::new(
                        coord.x.max(0.0).min(14980.0),
                        coord.y.max(0.0).min(14980.0),
                    ))
                })
            })
            .try_for_each(|walls| cdt.add_constraint_edges(walls, true))
            .unwrap();

        // TODO: to be explored
        // let hull = cdt.convex_hull().map(|e| e.to().position()).collect::<Vec<_>>();
        // cdt.add_constraint_edges(hull, true).unwrap();
        cdt
    }
}

pub struct MapTri {
    tri: NavMapTriangulation,
}

impl MapTri {
    pub fn new() -> Self {
        Self {
            tri: NavMapTriangulation {
                walls: rstar::RTree::new(),
                nav: rstar::RTree::new(),
            },
        }
    }
}

impl Pipe for MapTri {
    type Input = Vec<crate::mesh_mapper::Mesh>;

    type Output = ConstrainedDelaunayTriangulation<Point2<f64>>;

    type Error = crate::Error;

    #[tracing::instrument("delaunay", skip(self, input), fields(meshes=input.len()))]
    fn process(&mut self, input: Self::Input) -> Result<Option<Self::Output>, Self::Error> {
        let mut mw = geo::MultiPolygon::new(vec![]);
        for mesh in input {
            match mesh {
                crate::mesh_mapper::Mesh::Wall(wall) => {
                    self.tri.populate_walls(&wall.poly, wall.id);
                    mw.0.push(wall.poly);
                }
                crate::mesh_mapper::Mesh::Nav(_) => {}
                crate::mesh_mapper::Mesh::Unspecified(_) => {}
            }
        }

        debug!(bb = ?mw.bounding_rect());

        Ok(Some(self.tri.build()))
    }
}