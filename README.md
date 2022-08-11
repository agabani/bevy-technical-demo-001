# Bevy Technical Demo

```terminal
cargo sqlx prepare --database-url postgres://postgres:password@localhost:5432/bevy_technical_demo -- --bin bevy-technical-demo --features server
```

```terminal
$env:RUST_LOG="info,sqlx=warn"
cargo run --features "server"
```
