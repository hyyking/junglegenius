use libmap::svg::LineStringSampler;
use tracing::Level;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .init();

    let mut result = std::fs::File::create("map.json").unwrap();
    libmap::svg2geojson("map.svg", &mut result, LineStringSampler { rate: 32.0 }).unwrap();
}
