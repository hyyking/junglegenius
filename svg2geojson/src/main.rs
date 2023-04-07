fn main() {
    let mut result = std::fs::File::create("map.json").unwrap();
    svg2geojson::svg2geojson_filter_rgb("map2.svg".as_ref(), &mut result, 512.0, |rgb| {
       true
    })
    .unwrap();
}
