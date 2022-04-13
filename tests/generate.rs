use anyhow::Result;
use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use indoc::indoc;
use predicates::prelude::*;
use std::process::Command;
use toml::toml;

#[test]
fn generate_reports_error_when_templates_arg_missing() -> Result<()> {
    let package_dir = TempDir::new()?;

    Command::cargo_bin("cargo-about")?
        .current_dir(&package_dir)
        .arg("generate")
        .assert()
        .failure()
        // TODO: check for <TEMPLATE> string
        .stderr(predicate::str::is_match(
            r"required arguments were not provided:\s*<TEMPLATES>",
        )?);

    Ok(())
}

#[test]
fn generate_reports_error_when_manifest_absent() -> Result<()> {
    let package_dir = TempDir::new()?;

    Command::cargo_bin("cargo-about")?
        .current_dir(&package_dir)
        .arg("generate")
        .arg("my-about.hbs")
        .assert()
        .failure()
        .stderr(predicate::str::is_match(
            r"cargo manifest path '.*' does not exist",
        )?);

    Ok(())
}

#[test]
fn generate_reports_error_when_manifest_invalid() -> Result<()> {
    let package_dir = TempDir::new()?;
    let cargo_toml = package_dir.child("Cargo.toml");
    cargo_toml.touch()?;

    Command::cargo_bin("cargo-about")?
        .current_dir(&package_dir)
        .arg("generate")
        .arg("my-about.hbs")
        .assert()
        .failure()
        .stderr(predicate::str::contains("failed to parse manifest"));

    Ok(())
}

#[test]
fn generate_reports_error_when_template_path_invalid() -> Result<()> {
    let package_dir = TempDir::new()?;
    let cargo_toml = package_dir.child("Cargo.toml");
    cargo_toml.write_str(
        &toml::toml! {
            [package]
            name = "package"
            version = "0.0.0"
        }
        .to_string(),
    )?;
    package_dir.child("src/main.rs").touch()?;

    Command::cargo_bin("cargo-about")?
        .current_dir(&package_dir)
        .arg("generate")
        .arg("my-about.hbs")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "template(s) path my-about.hbs does not exist",
        ));

    Ok(())
}

#[test]
fn generate_empty_when_no_licenses() -> Result<()> {
    let package_dir = TempDir::new()?;
    package_dir.child("Cargo.toml").write_str(
        &(toml::toml! {
            [package]
            name = "package"
            version = "0.0.0"
        })
        .to_string(),
    )?;
    package_dir.child("src/main.rs").touch()?;
    package_dir.child("my-about.hbs").write_str(indoc! {"
        {{#each overview}}
        id: {{id}}
        name: {{name}}
        count: {{count}}
        {{/each}}
    "})?;

    Command::cargo_bin("cargo-about")?
        .current_dir(&package_dir)
        .arg("generate")
        .arg("my-about.hbs")
        .assert()
        .success()
        .stdout("\n");

    Ok(())
}

#[test]
fn generate_reports_error_when_foo() -> Result<()> {
    let package_dir = TempDir::new()?;
    package_dir.child("Cargo.toml").write_str(
        &toml::toml! {
            [package]
            name = "package"
            version = "0.0.0"
            license = "MIT"
        }
        .to_string(),
    )?;
    package_dir.child("src/main.rs").touch()?;
    package_dir.child("about.toml").write_str(
        &toml! {
            accepted = [ "MIT" ]
        }
        .to_string(),
    )?;
    package_dir.child("my-about.hbs").write_str(indoc! {"
        {{#each overview}}
        count: {{count}}
        name: {{name}}
        id: {{id}}
        {{/each}}

        {{#each licenses}}
        name: {{name}}
        id: {{id}}
        source_path: {{source_path}}
        text: {{text}}
        {{/each}}
    "})?;

    Command::cargo_bin("cargo-about")?
        .current_dir(&package_dir)
        .arg("generate")
        .arg("my-about.hbs")
        .assert()
        .success()
        .stdout(predicate::str::is_match(indoc! {r"
            count: 1
            name: MIT License
            id: MIT

            name: MIT License
            id: MIT
            source_path: 
            text: .*

        "})?);

    Ok(())
}
