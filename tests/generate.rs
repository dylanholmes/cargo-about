use anyhow::Result;
use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use indoc::indoc;
use predicates::prelude::*;
use std::process::Command;
use toml::toml;

#[test]
fn generate_fails_when_templates_arg_missing() -> Result<()> {
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
fn generate_fails_when_manifest_absent() -> Result<()> {
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
fn generate_fails_when_manifest_invalid() -> Result<()> {
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
fn generate_fails_when_template_path_invalid() -> Result<()> {
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
fn generate_fails_when_missing_accepted_field() -> Result<()> {
    let package_dir = TempDir::new()?;
    package_dir.child("Cargo.toml").write_str(
        &toml::toml! {
            [package]
            name = "package"
            version = "0.0.0"
        }
        .to_string(),
    )?;
    package_dir.child("src/main.rs").touch()?;
    package_dir.child("about.toml").touch()?;
    package_dir.child("my-about.hbs").touch()?;

    Command::cargo_bin("cargo-about")?
        .current_dir(&package_dir)
        .arg("generate")
        .arg("my-about.hbs")
        .assert()
        .failure()
        .stderr(predicates::str::contains("missing field `accepted`"));

    Ok(())
}

#[test]
fn generate_succeeds_with_warning_when_no_license_and_accepted_field_empty() -> Result<()> {
    let package_dir = TempDir::new()?;
    package_dir.child("Cargo.toml").write_str(
        &toml::toml! {
            [package]
            name = "package"
            version = "0.0.0"
        }
        .to_string(),
    )?;
    package_dir.child("src/main.rs").touch()?;
    package_dir.child("about.toml").write_str(
        &toml! {
            accepted = []
        }
        .to_string(),
    )?;
    package_dir.child("my-about.hbs").touch()?;

    Command::cargo_bin("cargo-about")?
        .current_dir(&package_dir)
        .arg("generate")
        .arg("my-about.hbs")
        .assert()
        .success()
        .stderr(predicates::str::contains(
            "unable to synthesize license expression for 'package 0.0.0': \
            no `license` specified, and no license files were found",
        ))
        .stdout("\n"); // TODO: Why does stdout contain a line feed character?

    Ok(())
}

#[test]
fn generate_fails_when_license_field_valid_and_accepted_field_empty() -> Result<()> {
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
            accepted = []
        }
        .to_string(),
    )?;
    package_dir.child("my-about.hbs").touch()?;

    Command::cargo_bin("cargo-about")?
        .current_dir(&package_dir)
        .arg("generate")
        .arg("my-about.hbs")
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "failed to satisfy license requirements",
        ))
        .stdout("");

    Ok(())
}

#[test]
fn generate_succeeds_with_warning_when_license_field_unknown() -> Result<()> {
    let package_dir = TempDir::new()?;
    package_dir.child("Cargo.toml").write_str(
        &toml::toml! {
            [package]
            name = "package"
            version = "0.0.0"
            license = "UNKNOWN"
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
    package_dir.child("my-about.hbs").touch()?;

    Command::cargo_bin("cargo-about")?
        .current_dir(&package_dir)
        .arg("generate")
        .arg("my-about.hbs")
        .assert()
        .success()
        .stderr(predicates::str::contains(
            "unable to parse license expression for 'package 0.0.0': UNKNOWN",
        ))
        .stdout("\n");

    Ok(())
}

#[test]
fn generate_writes_report_to_stdout_when_license_field_valid() -> Result<()> {
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
    package_dir.child("my-about.hbs").write_str(indoc! {r#"
        {{#each overview}}
        o,{{count}},{{name}},{{id}}
        {{/each}}
        {{#each licenses}}
        l,{{name}},{{id}},{{source_path}},{{text}}
        {{/each}}
    "#})?;

    let contains_mit_overview = predicates::str::contains("o,1,MIT License,MIT");
    let contains_mit_license = predicates::str::contains("l,MIT License,MIT,");

    Command::cargo_bin("cargo-about")?
        .current_dir(&package_dir)
        .arg("generate")
        .arg("my-about.hbs")
        .assert()
        .success()
        .stdout(contains_mit_overview)
        .stdout(contains_mit_license);

    Ok(())
}

#[test]
fn generate_writes_report_to_stdout_when_license_file_field_valid() -> Result<()> {
    let package_dir = TempDir::new()?;
    package_dir.child("Cargo.toml").write_str(
        &toml::toml! {
            [package]
            name = "package"
            version = "0.0.0"
            license_file = "MY_LICENSE"
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
    package_dir.child("my-about.hbs").write_str(indoc! {r#"
        {{#each overview}}
        o,{{count}},{{name}},{{id}}
        {{/each}}
        {{#each licenses}}
        l,{{name}},{{id}},{{source_path}},{{text}}
        {{/each}}
    "#})?;

    Command::cargo_bin("cargo-about")?
        .current_dir(&package_dir)
        .arg("generate")
        .arg("my-about.hbs")
        .assert()
        .success();

    panic!("not finished");
}

// Out of Scope
// - testing all SPDX Identifiers, that should be handled by the spdx crate which uses data from
// - testing all SPDX expressions
// Single Package -- License Field -- All SPDX Licenses are generated and any custom license file
// is used

// Single Package -- License File -- All SPDX Licenses are recovered and custom license file is
// used
