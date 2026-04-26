//! `[database]` URL resolution (Sprint 4).

use purwa_core::AppConfig;

#[test]
fn database_url_reads_toml_section() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("purwa.toml");
    std::fs::write(
        &path,
        r#"
[database]
url = "postgres://from-toml/db"
"#,
    )
    .unwrap();

    let cfg = AppConfig::load_with_file(Some(path.as_path())).unwrap();
    assert_eq!(
        cfg.database_url().as_deref(),
        Some("postgres://from-toml/db")
    );
}
