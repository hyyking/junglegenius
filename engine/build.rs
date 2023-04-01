fn main() {
    let mut result = std::fs::File::create("map.json").unwrap();

    svg2geojson::svg2geojson_filter_rgb("../map.svg".as_ref(), &mut result, |rgb| rgb.r > 25 && rgb.g > 25 && rgb.b > 25).unwrap();
}
