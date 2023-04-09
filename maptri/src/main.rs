use std::borrow::Borrow;

use geo::{
    euclidean_distance, Centroid, Contains, ConvexHull, CoordsIter, Intersects, LineString,
    LinesIter, MultiPolygon, Polygon, Relate, Within,
};
use geojson::FeatureCollection;
use rstar::primitives::GeomWithData;
use spade::{
    handles::FixedVertexHandle, ConstrainedDelaunayTriangulation, DelaunayTriangulation, Point2,
    Triangulation,
};

pub fn load_map(path: impl AsRef<std::path::Path>) -> geojson::FeatureCollection {
    let file = std::fs::File::open(path).unwrap();

    return FeatureCollection::try_from(geojson::GeoJson::from_reader(&file).unwrap()).unwrap();
}

fn main() {
    let map = load_map("map2.json");

    let mut cdt: ConstrainedDelaunayTriangulation<Point2<f64>> =
        spade::ConstrainedDelaunayTriangulation::new();

    let mut rstar = rstar::RTree::<GeomWithData<[f64; 2], FixedVertexHandle>>::new();

    let mut walls = vec![];

    let mut river = std::collections::HashMap::<String, Vec<Polygon>>::new();

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
            .and_then(|p| p.get("id").and_then(|s| s.as_str()));

        if let Some(groups) = groups {
            let mut insert = true;

            if groups.contains(&"bot_river_nav".to_string()) {
                let a = river
                    .entry("bot_river_nav".to_string())
                    .or_insert_with(Vec::new);
                a.push(poly.clone());
                insert = false;
            }

            if groups.contains(&"top_river_nav".to_string()) {
                let a = river
                    .entry("top_river_nav".to_string())
                    .or_insert_with(Vec::new);
                a.push(poly.clone());
                insert = false;
            }

            if insert && groups.contains(&"nav".to_string()) {
                dbg!(id);

                for line in poly.lines_iter() {
                    let start = line.start;
                    let end = line.end;
                    let a = cdt.insert(Point2::new(start.x, start.y)).unwrap();
                    let b = cdt.insert(Point2::new(end.x, end.y)).unwrap();

                    rstar.insert(GeomWithData::new([start.x, start.y], a));
                    rstar.insert(GeomWithData::new([end.x, end.y], b));
                }
            }

            if groups.contains(&"walls".to_string()) {
                walls.push(poly);
            }
        }
    }

    for river in river.into_values() {
        let poly = MultiPolygon::new(river).convex_hull();

        for line in poly.exterior().lines() {
            let start = line.start;
            let end = line.end;
            let a = cdt.insert(Point2::new(start.x, start.y)).unwrap();
            let b = cdt.insert(Point2::new(end.x, end.y)).unwrap();

            rstar.insert(GeomWithData::new([start.x, start.y], a));
            rstar.insert(GeomWithData::new([end.x, end.y], b));
        }
    }

    for poly in &walls {
        for line in poly
            .exterior()
            .lines()
            .chain(poly.interiors().iter().map(|a| a.lines()).flatten())
        {
            let start = line.start;
            let end = line.end;

            let from = rstar.nearest_neighbor(&[start.x, start.y]).unwrap().data;
            let to = rstar.nearest_neighbor(&[end.x, end.y]).unwrap().data;

            cdt.add_constraint(from, to);
        }
    }

    let mut features = vec![];

    let mut removed = 0;

    'a: for (i, face) in cdt.inner_faces().enumerate() {
        let [a, b, c] = face.vertices();

        let tri = geo::Triangle::new(
            geo::coord! { x: a.position().x, y: a.position().y },
            geo::coord! { x: b.position().x, y: b.position().y },
            geo::coord! { x: c.position().x, y: c.position().y },
        );

        for wall in &walls {
            if wall.contains(&tri.centroid()) {
                removed += 1;
                continue 'a;
            }
        }

        features.push(geojson::Feature {
            id: Some(geojson::feature::Id::String(format!("{i}"))),
            bbox: None,
            geometry: Some(tri.borrow().into()),
            properties: None,
            foreign_members: None,
        });
    }

    println!("{} walls removed {} triangles out of {}", walls.len(), removed, cdt.inner_faces().count());
    let mut f = std::fs::File::create("navmesh.json").unwrap();
    geojson::ser::to_feature_collection_writer(&mut f, &features).unwrap();
}

/*
fn main() {
    let navarea = load_map("maptri/map.json");
    let walls = load_map("engine/map.json");

    let nav = walls
        .features
        .clone()
        .into_iter()
        .map(|f| LineString::<f64>::try_from(f).unwrap());

    let mut dlt: DelaunayTriangulation<Point2<f64>> = spade::DelaunayTriangulation::new();

    for line in nav {
        for p in line.points() {
            dlt.insert(spade::Point2::new(p.x(), p.y())).unwrap();
        }
    }

    let mut features = vec![];

    'a: for (i, face) in dlt.inner_faces().enumerate() {
        let [a, b, c] = face.vertices();

        let tri = geo::Triangle::new(
            geo::coord! { x: a.position().x, y: a.position().y },
            geo::coord! { x: b.position().x, y: b.position().y },
            geo::coord! { x: c.position().x, y: c.position().y },
        );

        let walls = walls
            .features
            .iter()
            .map(|f| geo::Polygon::new(LineString::<f64>::try_from(f.clone()).unwrap(), vec![]));

        for wall in walls {
            // let intersection = tri.relate(&wall);
            if wall.contains(&tri) {
                continue 'a;
            }
        }
        features.push(geojson::Feature {
            id: Some(geojson::feature::Id::String(format!("{i}"))),
            bbox: None,
            geometry: Some(tri.borrow().into()),
            properties: None,
            foreign_members: None,
        });
    }

    let mut f = std::fs::File::create("navmesh.json").unwrap();
    geojson::ser::to_feature_collection_writer(&mut f, &features).unwrap();

    dbg!(dlt.num_inner_faces());
    dbg!(features.len());
}
 */
