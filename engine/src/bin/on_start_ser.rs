use engine::ecs::entity::TempSerEntity;

fn main() {
    let (_, store) = engine::MinimapEngine::init();

    let turrets = std::fs::File::create("turrets.json").unwrap();
    serde_json::ser::to_writer_pretty(
        turrets,
        &store.turrets().map(TempSerEntity::new).collect::<Vec<_>>(),
    )
    .unwrap();
}
