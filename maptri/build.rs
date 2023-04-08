use svg2geojson::sampler::PointSampler;

fn main() {
    println!("cargo:rerun-if-changed=../map.svg");
    println!("cargo:rerun-if-changed=map.json");
    println!("cargo:rerun-if-changed={}", file!());

    let mut result = std::fs::File::create("map.json").unwrap();
    
    svg2geojson::svg2geojson("../map.svg", &mut result, PointSampler { rate: 512.0 }).unwrap();
}
