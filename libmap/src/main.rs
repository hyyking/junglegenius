use libmap::svg::LineStringSampler;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("libmap=trace,geo=warn")
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::NONE)
        .init();

    let mut result = std::fs::File::create("map.json").unwrap();
    libmap::svg2geojson("map.svg", &mut result, LineStringSampler { rate: 32.0 }).unwrap();
}
