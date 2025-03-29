use std::process::Command;
#[test]
fn preflight_validates_without_database_and_rejects_fake_national_coverage() {
    let check = |extra: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_seed_wilayah"))
            .args(["--file", "tests/fixtures/wilayah.test.json", "--check-only"])
            .args(extra)
            .env("DATABASE_URL", "intentionally-not-a-database")
            .output()
            .unwrap()
    };
    let valid = check(&["--expected-provinces", "1", "--expected-cities", "2"]);
    assert!(
        valid.status.success(),
        "{}",
        String::from_utf8_lossy(&valid.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&valid.stdout).unwrap();
    assert_eq!(report["imported"], false);
    assert_eq!(report["cities"], 2);
    let invalid = check(&[]);
    assert!(!invalid.status.success());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("Dataset coverage mismatch"));
}
