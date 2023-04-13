use libmap::sampler::PointSampler;


fn main() {
    let mut result = std::fs::File::create("map2.json").unwrap();
    libmap::svg2geojson("map2.svg", &mut result, PointSampler { rate: 32.0 })
    .unwrap();
}