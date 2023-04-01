fn main() {
    println!("cargo:rerun-if-changed=../map.svg");
    println!("cargo:rerun-if-changed=map.json");
    println!("cargo:rerun-if-changed={}", file!());

    let mut result = std::fs::File::create("map.json").unwrap();
    svg2geojson::svg2geojson_filter_rgb("../map.svg".as_ref(), &mut result, 128.0, |rgb| {
        rgb.r <= 16 && rgb.g <= 24 && rgb.b <= 18
    })
    .unwrap();
}
