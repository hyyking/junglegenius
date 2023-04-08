use svg2geojson::sampler::PointSampler;


fn main() {
    let mut result = std::fs::File::create("map2.json").unwrap();
    svg2geojson::svg2geojson("map2.svg", &mut result, PointSampler { rate: 128.0 })
    .unwrap();
}
