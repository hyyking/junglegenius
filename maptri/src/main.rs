#![feature(iterator_try_collect)]

use std::{borrow::Borrow, collections::HashSet, io::Write};

use geo::{
    ClosestPoint, ConvexHull, CoordsIter, EuclideanDistance, Extremes, LineString, Point, Polygon,
    RemoveRepeatedPoints,
};
use geojson::{FeatureCollection, Geometry};
use maptri::cvt::{poly_from_voronoi_face, CentralVoronoiTesselation};
use rstar::{primitives::GeomWithData, PointDistance};
use spade::{ConstrainedDelaunayTriangulation, Point2, RefinementParameters, Triangulation};

pub fn load_map(path: impl AsRef<std::path::Path>) -> geojson::FeatureCollection {
    let file = std::fs::File::open(path).unwrap();

    return FeatureCollection::try_from(geojson::GeoJson::from_reader(&file).unwrap()).unwrap();
}
use rayon::prelude::*;

struct Wall(geo::Polygon, String);
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

struct NavMapTriangulation {
    walls: rstar::RTree<Wall>,
    nav: rstar::RTree<[f64; 2]>,
}

impl NavMapTriangulation {
    fn populate_walls(&mut self, wall: &Polygon, id: String) -> isize {
        const CLOSE: f64 = 16.0 * 16.0;

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
        /*
        for &[x, y] in self.nav.iter() {
            if x == 0.0 || y == 0.0 || x == 14930.0 || y == 14930.0 {
                continue;
            }
            let p = spade::mitigate_underflow(Point2::new(x, y));
            points.insert(GeomWithData::new([p.x, p.y], cdt.insert(p).unwrap()));
        }
         */

        if let Some(Wall(top_left, _)) = self.walls.locate_at_point(&[0.0, 0.0]) {
            let rect = top_left.extremes().unwrap();
            let p1 = spade::mitigate_underflow(Point2::new(rect.y_max.coord.x, rect.y_max.coord.y));
            let p2 = spade::mitigate_underflow(Point2::new(rect.x_max.coord.x, rect.x_max.coord.y));

            let v1 = cdt.insert(p1).unwrap();
            points.insert(GeomWithData::new([p1.x, p1.y], v1));

            let v2 = cdt.insert(p2).unwrap();
            points.insert(GeomWithData::new([p2.x, p2.y], v2));
        } else {
            let top_left = cdt.insert(Point2::new(0.0, 0.0)).unwrap();
            points.insert(GeomWithData::new([0.0, 0.0], top_left));
        }

        if let Some(Wall(top_right, _)) = self.walls.locate_at_point(&[14980.0, 0.0]) {
            let rect = top_right.extremes().unwrap();
            let p1 = spade::mitigate_underflow(Point2::new(rect.y_max.coord.x, rect.y_max.coord.y));
            let p2 = spade::mitigate_underflow(Point2::new(rect.x_max.coord.x, rect.x_max.coord.y));

            let v1 = cdt.insert(p1).unwrap();
            points.insert(GeomWithData::new([p1.x, p1.y], v1));

            let v2 = cdt.insert(p2).unwrap();
            points.insert(GeomWithData::new([p2.x, p2.y], v2));
        } else {
            let top_right = cdt.insert(Point2::new(14980.0, 0.0)).unwrap();
            points.insert(GeomWithData::new([14980.0, 0.0], top_right));
        }

        if let Some(Wall(bot_right, _)) = self.walls.locate_at_point(&[14980.0, 14980.0]) {
            let rect = bot_right.extremes().unwrap();
            let p1 = spade::mitigate_underflow(Point2::new(rect.y_max.coord.x, rect.y_max.coord.y));
            let p2 = spade::mitigate_underflow(Point2::new(rect.x_max.coord.x, rect.x_max.coord.y));

            let v1 = cdt.insert(p1).unwrap();
            points.insert(GeomWithData::new([p1.x, p1.y], v1));

            let v2 = cdt.insert(p2).unwrap();
            points.insert(GeomWithData::new([p2.x, p2.y], v2));
        } else {
            let bot_right = cdt.insert(Point2::new(14980.0, 14980.0)).unwrap();
            points.insert(GeomWithData::new([14980.0, 14980.0], bot_right));
        }

        if let Some(Wall(bot_right, _)) = self.walls.locate_at_point(&[0.0, 14980.0]) {
            let rect = bot_right.extremes().unwrap();
            let p1 = spade::mitigate_underflow(Point2::new(rect.y_max.coord.x, rect.y_max.coord.y));
            let p2 = spade::mitigate_underflow(Point2::new(rect.x_max.coord.x, rect.x_max.coord.y));

            let v1 = cdt.insert(p1).unwrap();
            points.insert(GeomWithData::new([p1.x, p1.y], v1));

            let v2 = cdt.insert(p2).unwrap();
            points.insert(GeomWithData::new([p2.x, p2.y], v2));
        } else {
            let bot_right = cdt.insert(Point2::new(0.0, 14980.0)).unwrap();
            points.insert(GeomWithData::new([0.0, 14980.0], bot_right));
        }
        let hull = points
            .into_iter()
            .map(|p| geo::coord! {x: p.geom()[0], y: p.geom()[1]})
            .collect::<LineString>()
            .convex_hull();
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

fn main() {
    let mut tri = NavMapTriangulation {
        walls: rstar::RTree::new(),
        nav: rstar::RTree::new(),
    };

    let map = load_map("map2.json");

    for feature in map.features {
        let groups = feature
            .properties
            .as_ref()
            .unwrap()
            .get("properties")
            .and_then(|p| {
                p.as_object()
                    .and_then(|m| m.get("groups").and_then(|v| v.as_array()))
            })
            .map(|groups| {
                groups
                    .iter()
                    .map(|v| v.as_str().unwrap().to_string())
                    .collect::<Vec<_>>()
            });

        let poly = geo::Polygon::<f64>::try_from(feature.geometry.unwrap()).unwrap();

        let id = feature
            .properties
            .as_ref()
            .and_then(|p| p.get("id").and_then(|s| s.as_str()))
            .unwrap_or_default()
            .to_string();

        if let Some(groups) = groups {
            if groups.contains(&"walls".to_string()) {
                let new = tri.populate_walls(&poly, id.clone());
                println!("WALL {id} | {new} vertices simplified");
            }
        }
    }

    let mut cdt = tri.build();

    let refr = cdt.refine(
        RefinementParameters::new()
            .keep_constraint_edges()
            .exclude_outer_faces(&cdt)
            .with_min_required_area(128.0 * 128.0),
    );
    println!("BUILD Refinement | complete {}", refr.refinement_complete);
    println!("inner faces {}", cdt.inner_faces().count());
    println!("excluded faces: {}", refr.excluded_faces.len());

    let cdt = cdt.cvt_lloyds_algorithm(8.0).unwrap();

    let mut s = flexbuffers::FlexbufferSerializer::new();
    use serde::ser::Serialize;
    cdt.serialize(&mut s).unwrap();

    let mut f = std::fs::File::create("nashmesh.flat").unwrap();
    f.write_all(s.view()).unwrap();

    let features = voronoi_faces_geom(cdt, &tri.walls)
        .unwrap()
        .into_iter()
        .enumerate()
        .map(|(i, poly)| geojson::Feature {
            id: Some(geojson::feature::Id::String(format!("{i}"))),
            bbox: None,
            geometry: Some(poly),
            properties: None,
            foreign_members: None,
        })
        .collect::<Vec<_>>();

    println!("BUILD: writing {} poly", features.len());
    let mut f = std::fs::File::create("navmesh.json").unwrap();
    geojson::ser::to_feature_collection_writer(&mut f, &features).unwrap();
    // write_cdt(cdt, &tri.walls, &refr.excluded_faces);
}

fn write_cdt(
    cdt: ConstrainedDelaunayTriangulation<Point2<f64>>,
    _walls: &rstar::RTree<Wall>,
    excluded: &HashSet<spade::handles::FixedFaceHandle<spade::handles::InnerTag>>,
) {
    let mut features = Vec::with_capacity(cdt.inner_faces().count());

    'a: for (i, face) in cdt.inner_faces().enumerate() {
        if excluded.contains(&face.fix()) {
            continue;
        }
        let mut exterior = LineString(Vec::new());

        let [a, b, c] = face.adjacent_edges();

        for position in a
            .vertices()
            .into_iter()
            .chain(b.vertices().into_iter())
            .chain(c.vertices().into_iter())
        {
            let a = position.data();
            let coord = geo::coord! { x: a.x, y: a.y };
            exterior.0.push(coord);
        }

        exterior.remove_repeated_points_mut();
        exterior.close();
        let tri = geo::Polygon::new(exterior, vec![]);

        features.push(geojson::Feature {
            id: Some(geojson::feature::Id::String(format!("{i}"))),
            bbox: None,
            geometry: Some(tri.borrow().into()),
            properties: None,
            foreign_members: None,
        });
    }

    println!("BUILD: writing {} poly", features.len());
    let mut f = std::fs::File::create("navmesh.json").unwrap();
    geojson::ser::to_feature_collection_writer(&mut f, &features).unwrap();
}

fn voronoi_faces_geom(
    cdt: ConstrainedDelaunayTriangulation<Point2<f64>>,
    walls: &rstar::RTree<Wall>,
) -> Result<Vec<Geometry>, ()> {
    cdt.vertices()
        .collect::<Vec<_>>()
        .into_par_iter()
        .map(|v| v.as_voronoi_face())
        .map(poly_from_voronoi_face)
        .map(|poly| poly.map(|poly| fit_poly_to_walls(poly, walls)))
        .map(|poly| poly.map(|p| Geometry::from(&p)))
        .collect::<Result<Vec<_>, ()>>()
}

fn fit_poly_to_walls(poly: Polygon, walls: &rstar::RTree<Wall>) -> Polygon {
    let coords = poly.coords_iter().filter_map(|coord| {
        if let Some(Wall(wall, _)) = walls.locate_at_point(&[coord.x, coord.y]) {
            let closest = wall.exterior().closest_point(&coord.into());
            match closest {
                geo::Closest::Intersection(p) => Some(p.into()),
                geo::Closest::SinglePoint(p) => Some(p.into()),
                geo::Closest::Indeterminate => None,
            }
        } else {
            Some(coord)
        }
    });
    geo::Polygon::new(coords.collect::<LineString>(), vec![])
}
