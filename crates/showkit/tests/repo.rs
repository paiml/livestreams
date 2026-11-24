//! Repo-level facts: every show passes its shapes, and the shapes file states the bounds the code enforces.
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn every_show_dir_has_facts_that_pass() {
    let mut n = 0;
    for e in std::fs::read_dir(root().join("shows")).unwrap() {
        let p = e.unwrap().path();
        if !p.is_dir() {
            continue;
        }
        let f = p.join("facts.toml");
        assert!(f.exists(), "{} has no facts.toml", f.display());
        assert_eq!(
            showkit::run(&["check".into(), f.display().to_string()]),
            0,
            "{}",
            f.display()
        );
        n += 1;
    }
    assert!(n >= 2, "backfill missing: {n} shows");
}

#[test]
fn shapes_file_names_every_fact_and_bound() {
    let ttl = std::fs::read_to_string(root().join("shapes/show.shacl.ttl")).unwrap();
    for want in [
        "show:title",
        "show:duration_s",
        "sh:minInclusive 60",
        "sh:maxInclusive 3600",
        "show:loudness_lufs",
        "sh:minInclusive -24",
        "sh:maxInclusive -12",
        "show:true_peak_dbfs",
        "sh:maxInclusive -1 ",
        "show:max_black_run_s",
        "sh:maxInclusive 0.5",
        "show:refusal_text",
        "sh:maxInclusive 0 ",
    ] {
        assert!(ttl.contains(want), "shapes file lacks {want}");
    }
}
