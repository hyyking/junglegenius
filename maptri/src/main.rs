use std::borrow::Borrow;

use geo::{Contains, Intersects, LineString, Relate, Within};
use geojson::FeatureCollection;
use spade::{DelaunayTriangulation, Point2, Triangulation};

pub fn load_map(path: impl AsRef<std::path::Path>) -> geojson::FeatureCollection {
    let file = std::fs::File::open(path).unwrap();

    return FeatureCollection::try_from(geojson::GeoJson::from_reader(&file).unwrap()).unwrap();
}

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
