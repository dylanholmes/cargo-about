use anyhow::Result;
use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use std::{path::PathBuf, process::Command};
//use predicates::prelude::*;

fn test_crate_path(crate_name: &str) -> Result<PathBuf> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("resources");
    path.push(crate_name);
    Ok(path)
}

#[test]
fn init_silently_does_nothing_in_empty_crate() -> Result<()> {
    Command::cargo_bin("cargo-about")?
        .current_dir(test_crate_path("empty-package")?)
        .arg("init")
        .assert()
        .success()
        .stdout("");
    Ok(())
}

#[test]
fn init_works() -> Result<()> {
    let package_dir = TempDir::new()?;
    let cargo_toml = package_dir.child("Cargo.toml");
    cargo_toml.touch()?;

    Command::cargo_bin("cargo-about")?
        .current_dir(&package_dir)
        .arg("init")
        .assert()
        .failure()
        .code(1)
        .stdout("");

    Ok(())
}
