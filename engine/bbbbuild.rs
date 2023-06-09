use libmap::svg::LineStringSampler;

fn main() {
    println!("cargo:rerun-if-changed=../map.svg");
    println!("cargo:rerun-if-changed=map.json");
    println!("cargo:rerun-if-changed={}", file!());

    let mut result = std::fs::File::create("map.json").unwrap();
    libmap::svg2geojson(
        "../map.svg",
        &mut result,
        LineStringSampler { rate: 128.0 },
    )
    .unwrap();
}
