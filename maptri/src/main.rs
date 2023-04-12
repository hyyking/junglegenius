#![feature(iterator_try_collect)]

use std::{borrow::Borrow, cmp::min_by_key, collections::HashSet, f32::consts::E};

use geo::{
    coordinate_position::CoordPos, euclidean_distance, line_intersection::line_intersection,
    BoundingRect, Centroid, ClosestPoint, ConcaveHull, Contains, ConvexHull, CoordsIter,
    EuclideanDistance, Extremes, Intersects, IsConvex, Line, LineLocatePoint, LineString,
    LinesIter, MultiLineString, MultiPolygon, Point, Polygon, Relate, RemoveRepeatedPoints, Rotate,
    Simplify, Within,
};
use geojson::{FeatureCollection, Geometry};
use num_traits::{Float, FromPrimitive};
use rstar::{primitives::GeomWithData, PointDistance};
use spade::{
    handles::{FixedVertexHandle, VertexHandle, VoronoiFace},
    ConstrainedDelaunayTriangulation, HasPosition, Point2, RefinementParameters, Triangulation,
};

pub fn load_map(path: impl AsRef<std::path::Path>) -> geojson::FeatureCollection {
    let file = std::fs::File::open(path).unwrap();

    return FeatureCollection::try_from(geojson::GeoJson::from_reader(&file).unwrap()).unwrap();
}
use rayon::prelude::*;
const REDUNDANCY_RADIUS: f64 = 16.0;
const CLOSE: f64 = REDUNDANCY_RADIUS * REDUNDANCY_RADIUS;

struct Poly(geo::Polygon, String);
impl PointDistance for Poly {
    fn distance_2(
        &self,
        point: &<Self::Envelope as rstar::Envelope>::Point,
    ) -> <<Self::Envelope as rstar::Envelope>::Point as rstar::Point>::Scalar {
        self.0
            .euclidean_distance(&geo::point! {x: point[0], y: point[1]})
    }
}

impl rstar::RTreeObject for Poly {
    type Envelope = oobb::OOBB<f64>;

    fn envelope(&self) -> Self::Envelope {
        oobb::OOBB::from_polygon(self.0.clone())
    }
}

struct NavMapTriangulation {
    walls: rstar::RTree<Poly>,
    nav: rstar::RTree<[f64; 2]>,
}

impl NavMapTriangulation {
    fn populate_nav(&mut self, nav: &Polygon, id: String) {
        let prev = self.nav.iter().count();
        for coord in nav.coords_iter() {
            match self
                .nav
                .nearest_neighbor_iter_with_distance_2(&[
                    coord.x.max(0.0).min(14980.0),
                    coord.y.max(0.0).min(14980.0),
                ])
                .find(|(_, d)| *d < CLOSE)
            {
                Some((intersect, _)) => {
                    self.nav.remove(&intersect.clone()).unwrap();
                }
                None => self
                    .nav
                    .insert([coord.x.max(0.0).min(14980.0), coord.y.max(0.0).min(14980.0)]),
            }
        }
        let after = self.nav.iter().count();

        let points = nav.coords_iter().count();
        println!(
            "INSERT NAV {id:16}: {points:4} points, {:4} redundant",
            points - (after - prev)
        );
    }

    fn populate_walls(&mut self, wall: &Polygon, id: String) -> isize {
        const CLOSE: f64 = 32.0 * 32.0;

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
        let poly = Poly(Polygon::new(ext, int), id);
        let new = wall.coords_count() as isize - poly.0.coords_count() as isize;
        self.walls.insert(poly);
        new
    }

    fn build(&self) -> spade::ConstrainedDelaunayTriangulation<Point2<f64>> {
        let mut cdt = spade::ConstrainedDelaunayTriangulation::<Point2<f64>>::new();

        // let top_left = cdt.insert(Point2::new(0.0, 0.0)).unwrap();
        // let bot_right = cdt.insert(Point2::new(14980.0, 14980.0)).unwrap();
        cdt.insert(Point2::new(0.0, 14980.0)).unwrap();
        cdt.insert(Point2::new(14980.0, 0.0)).unwrap();

        let mut points = rstar::RTree::new();

        for &[x, y] in self.nav.iter() {
            if x == 0.0 || y == 0.0 || x == 14930.0 || y == 14930.0 {
                continue;
            }
            let p = spade::mitigate_underflow(Point2::new(x, y));
            points.insert(GeomWithData::new([p.x, p.y], cdt.insert(p).unwrap()));
        }

        if let Some(Poly(top_bound, id)) = self.walls.locate_at_point(&[0.0, 0.0]) {
            assert_eq!(id, "top_bound");
            let rect = top_bound.extremes().unwrap();
            let p1 = spade::mitigate_underflow(Point2::new(rect.y_max.coord.x, rect.y_max.coord.y));
            let p2 = spade::mitigate_underflow(Point2::new(rect.x_max.coord.x, rect.x_max.coord.y));

            let v1 = cdt.insert(p1).unwrap();
            points.insert(GeomWithData::new([p1.x, p1.y], v1));

            let v2 = cdt.insert(p2).unwrap();
            points.insert(GeomWithData::new([p2.x, p2.y], v2));
        }

        if let Some(Poly(bot_bound, id)) = self.walls.locate_at_point(&[14980.0, 14980.0]) {
            assert_eq!(id, "bot_bound");
            let rect = bot_bound.extremes().unwrap();
            let p1 = spade::mitigate_underflow(Point2::new(rect.y_max.coord.x, rect.y_max.coord.y));
            let p2 = spade::mitigate_underflow(Point2::new(rect.x_max.coord.x, rect.x_max.coord.y));

            let v1 = cdt.insert(p1).unwrap();
            points.insert(GeomWithData::new([p1.x, p1.y], v1));

            let v2 = cdt.insert(p2).unwrap();
            points.insert(GeomWithData::new([p2.x, p2.y], v2));
        }
        /*
                for Poly(wall, id) in self.walls.iter() {
                    if id == "top_bound" {

                        // cdt.add_constraint(v1, top_left);
                        // cdt.add_constraint(v2, top_left);
                    }
                    if id == "bot_bound" {
                        let rect = wall.extremes().unwrap();
                        let p1 =
                            spade::mitigate_underflow(Point2::new(rect.y_min.coord.x, rect.y_min.coord.y));
                        let p2 =
                            spade::mitigate_underflow(Point2::new(rect.x_min.coord.x, rect.x_min.coord.y));

                        let v1 = cdt.insert(p1).unwrap();
                        points.insert(GeomWithData::new([p1.x, p1.y], v1));

                        let v2 = cdt.insert(p2).unwrap();
                        points.insert(GeomWithData::new([p2.x, p2.y], v2));

                        // cdt.add_constraint(v1, bot_right);
                        // cdt.add_constraint(v2, bot_right);
                    }
                }
        */

        let hull = points
            .iter()
            .map(|p| geo::coord! {x: p.geom()[0], y: p.geom()[1]})
            .chain(Some(geo::coord! { x: 0.0, y: 14980.0 }))
            .chain(Some(geo::coord! { x: 14980.0, y: 0.0 }))
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

        let mut inserted = cdt.num_vertices();

        for Poly(wall, id) in self.walls.iter() {
            let prev_inserted = inserted;
            let mut local_constraints = cdt.num_constraints();

            if id == "bot_bound" || id == "top_bound" {
                continue;
            }

            cdt.add_constraint_edges(
                wall.coords_iter().map(|coord| {
                    spade::mitigate_underflow(Point2::new(
                        coord.x.max(0.0).min(14980.0),
                        coord.y.max(0.0).min(14980.0),
                    ))
                }),
                true,
            )
            .unwrap();

            inserted = cdt.num_vertices();
            local_constraints = cdt.num_constraints() - local_constraints;

            let total_lines = wall.lines_iter().count();
            let s = inserted - prev_inserted;
            let cons_percentage = (local_constraints as f32 / total_lines as f32) * 100.0;
            println!(
                    "BUILD WALL {id:>16} ({:>4} ext|{:<4} int): {total_lines:4} wall lines {s:4} vertices inserted, {cons_percentage}% constrained ({local_constraints:>4})",
                    wall.exterior().lines_iter().count(),
                    wall.interiors().iter().map(|i| format!("{}", i.lines_iter().count())).collect::<Vec<String>>().join("-"),
                );

            if cons_percentage < 40.0 {
                for line in wall.lines_iter() {
                    let start = cdt.locate_vertex(Point2::new(line.start.x, line.start.y));
                    let end = cdt.locate_vertex(Point2::new(line.end.x, line.end.y));

                    let constraint = matches!((start, end), (Some(a), Some(b)) if cdt.exists_constraint(a.fix(), b.fix()));

                    if constraint {
                        println!("K : {:4.4?} is constrained", line);
                    } else {
                        println!("X : {:4.4?} is NOT constrained", line);
                    }
                }
            }
        }

        let total_lines = self
            .walls
            .iter()
            .map(|p| p.0.lines_iter().count())
            .sum::<usize>();

        let constraints = cdt.num_constraints();
        println!(
            "TOTAL WALL: {total_lines} wall lines {inserted} vertices inserted, {}% constrained ({constraints})",
            (constraints as f32 / total_lines as f32) * 100.0
        );
        cdt
    }
}

fn main() {
    let mut tri = NavMapTriangulation {
        walls: rstar::RTree::new(),
        nav: rstar::RTree::new(),
    };

    let map = load_map("map2.json");

    let mut river = std::collections::HashMap::<String, Vec<Polygon>>::new();

    let mut walls = vec![];
    println!("----------------------- NAV ------------------------");
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
            if groups.contains(&"bot_river_nav".to_string()) {
                let a = river
                    .entry("bot_river_nav".to_string())
                    .or_insert_with(Vec::new);
                a.push(poly.clone());
                continue;
            }

            if groups.contains(&"top_river_nav".to_string()) {
                let a = river
                    .entry("top_river_nav".to_string())
                    .or_insert_with(Vec::new);
                a.push(poly.clone());
                continue;
            }

            if groups.contains(&"nav".to_string()) {
                // tri.populate_nav(&poly, id.clone());
            }

            if groups.contains(&"walls".to_string()) {
                walls.push((poly.clone(), id.clone()));
            }
        }
    }

    for (id, river) in river.into_iter() {
        let poly = MultiPolygon::new(river).convex_hull();
        // tri.populate_nav(&poly, id);
    }

    println!("----------------------- WALL -----------------------");
    let new = walls
        .into_iter()
        .map(|(poly, id)| tri.populate_walls(&poly, id))
        .sum::<isize>();
    println!("{new} vertices simplified");

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

    let features = central_voronoi_faces(cdt, &tri.walls)
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
    mut cdt: ConstrainedDelaunayTriangulation<Point2<f64>>,
    walls: &rstar::RTree<Poly>,
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

pub trait CentralVoronoiTesselation {
    fn cvt_lloyds_algorithm(self, max_distance2: f64) -> Result<Self, ()>
    where
        Self: Sized;
}

impl CentralVoronoiTesselation for ConstrainedDelaunayTriangulation<Point2<f64>> {
    fn cvt_lloyds_algorithm(mut self, max_distance2: f64) -> Result<Self, ()> {
        // let mut move_to = vec![];
        let mut c = 0.0;
        loop {
            use rayon::prelude::*;

            let move_to = self
                .vertices()
                .map(|v| (v, v.data().clone()))
                .collect::<Vec<_>>()
                .into_par_iter()
                .map(|(handle, vertex)| (handle.as_voronoi_face(), vertex))
                .map(|(handle, vertex)| (vertex, poly_from_voronoi_face(handle)))
                .filter_map(|(vertex, face)| {
                    face.map(|face| face.centroid().map(|c| (vertex, Point2::new(c.x(), c.y()))))
                        .transpose()
                })
                .filter(|points| points.map(|(p, c)| p.distance_2(c) > max).unwrap_or(false))
                .collect::<Result<Vec<_>, ()>>()?;

            let max_move = move_to.iter().fold(0.0, |a, b| a.max(b.0.distance_2(b.1)));

            if move_to.is_empty() || c == max_move {
                break Ok(self);
            }

            for (v, center) in move_to.into_iter() {
                // println!("move {:?} to {center:?}", v);
                self.remove(self.locate_vertex(v).unwrap().fix());
                self.insert(center).unwrap();
            }
            dbg!(max_move);
            c = max_move;
        }
    }
}

fn central_voronoi_faces(
    cdt: ConstrainedDelaunayTriangulation<Point2<f64>>,
    walls: &rstar::RTree<Poly>,
) -> Result<Vec<Geometry>, ()> {
    
    const LARGE: f64 = 10000.0 * 10000.0;

    let cdt = cdt.cvt_lloyds_algorithm(32.0)?;
    
    
    

    cdt.vertices()
        .collect::<Vec<_>>()
        .into_par_iter()
        .map(|v| v.as_voronoi_face())
        .map(poly_from_voronoi_face)
        .map(|poly| poly.map(|poly| fit_poly_to_walls(poly, walls)))
        .filter(|poly| {
            if let Ok(Some(extrs)) = poly.as_ref().map(|poly| poly.extremes()) {
                let dx = extrs.x_max.coord.distance_2(&extrs.x_min.coord);
                let dy = extrs.y_max.coord.distance_2(&extrs.y_min.coord);
                dx + dy < LARGE
            } else {
                true
            }
        })
        .filter(|poly| {
            if let Ok(Some((poly, Poly(wall, _)))) = poly.as_ref().map(|poly| {
                poly.centroid()
                    .and_then(|center| walls.locate_at_point(&[center.x(), center.y()]))
                    .map(|wall| (poly, wall))
            }) {
                !wall.intersects(poly)
            } else {
                true
            }
        })
        .map(|poly| poly.map(|p| Geometry::from(&p)))
        .collect::<Result<Vec<_>, ()>>()

    /*
    for (i, v) in cdt.vertices().enumerate() {
        let poly = fit_poly_to_walls(poly_from_voronoi_face(v.as_voronoi_face())?, walls);

        // TODO: remove this fix, but idk (removes weird tall polys)
        if let Some(a) = poly.extremes() {
            if a.x_max.coord.distance_2(&a.x_min.coord) + a.y_max.coord.distance_2(&a.y_min.coord)
                > 10000.0 * 10000.0
            {
                dbg!(i);
                continue;
            }
        }

        if let Some(Poly(wall, _)) = poly
            .centroid()
            .and_then(|center| walls.locate_at_point(&[center.x(), center.y()]))
        {
            if wall.intersects(&poly) {
                continue;
            }
        }

        features.push(poly.borrow().into());
    }
    Ok(features)
     */
}

fn fit_poly_to_walls(poly: Polygon, walls: &rstar::RTree<Poly>) -> Polygon {
    geo::Polygon::new(
        poly.coords_iter()
            .filter_map(|coord| {
                if let Some(Poly(wall, _)) = walls.locate_at_point(&[coord.x, coord.y]) {
                    let closest = wall.exterior().closest_point(&coord.into());
                    match closest {
                        geo::Closest::Intersection(p) => Some(p.into()),
                        geo::Closest::SinglePoint(p) => Some(p.into()),
                        geo::Closest::Indeterminate => None,
                    }
                } else {
                    Some(coord)
                }
            })
            .collect::<LineString>(),
        vec![],
    )
}

pub fn poly_from_voronoi_face<DE, UE, F, T>(
    face: VoronoiFace<'_, T, DE, UE, F>,
) -> Result<geo::Polygon<<T as spade::HasPosition>::Scalar>, ()>
where
    T: spade::HasPosition,
    <T as spade::HasPosition>::Scalar:
        spade::SpadeNum + num_traits::float::Float + PartialOrd + FromPrimitive,
{
    let mut exterior = LineString(Vec::with_capacity(face.adjacent_edges().count()));

    for edge in face.adjacent_edges() {
        let vertex = edge.to().position().or(edge.from().position());

        if let Some(pos) = vertex {
            let coord = geo::coord! {
            x: <T::Scalar>::max(<T::Scalar as Float>::min(pos.x, T::Scalar::from(14980.0)), T::Scalar::from(0.0)), y: pos.y.min(T::Scalar::from(14980.0)).max(T::Scalar::from(0.0))};

            exterior.0.push(coord);
        } else {
            return Err(());
        }
    }

    exterior.remove_repeated_points_mut();
    exterior.close();
    Ok(geo::Polygon::new(exterior, vec![]))
}
