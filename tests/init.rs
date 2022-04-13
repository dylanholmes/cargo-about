use anyhow::Result;
use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use predicates::prelude::*;
use std::path::PathBuf;
use std::process::Command;

//struct TestPackage {
//    dir: TempDir,
//}
//
//impl TestPackage {
//    pub fn new() -> Result<Self> {
//        Ok(Self {
//            dir: TempDir::new()?,
//        })
//    }
//
//    pub cargo_toml(&mut self, content: &str) -> Result<&mut Self> {
//    }
//
//    pub fn dir(&self) -> &TempDir {
//        &self.dir
//    }
//}

#[test]
fn init_reports_error_when_manifest_absent() -> Result<()> {
    let package_dir = TempDir::new()?;
    Command::cargo_bin("cargo-about")?
        .arg("init")
        .current_dir(&package_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("could not find `Cargo.toml`"));
    Ok(())
}

#[test]
fn init_reports_error_when_manifest_empty() -> Result<()> {
    let package_dir = TempDir::new()?;
    let cargo_toml = package_dir.child("Cargo.toml");
    cargo_toml.touch()?;

    Command::cargo_bin("cargo-about")?
        .current_dir(&package_dir)
        .arg("init")
        .assert()
        .failure()
        .stderr(predicate::str::contains("failed to parse manifest"));

    Ok(())
}

// TODO: Split this into two tests after refactoring the test setup code
// - one test for "generates config"
// - one test for "generates template"
// - Similarly for other tests in this file
// Should we validate the generated content? Maybe if it is easy
// to compare with the source file... otherwise no.
#[test]
fn init_generates_config_when_config_and_template_absent() -> Result<()> {
    // Create a minimal valid rust package.
    let package_dir = TempDir::new()?;
    package_dir.child("src/main.rs").touch()?;
    let cargo_toml = package_dir.child("Cargo.toml");
    cargo_toml.touch()?;
    let cargo_toml_content = toml::toml! {
        [package]
        name = "package"
        version = "0.0.0"
    };
    cargo_toml.write_str(&cargo_toml_content.to_string())?;
    let config = package_dir.child("about.toml");
    let template = package_dir.child("about.hbs");

    // Validate the preconditions.
    config.assert(predicate::path::missing());
    template.assert(predicate::path::missing());

    // Invoke into to generate the default `about.hbs` template.
    Command::cargo_bin("cargo-about")?
        .current_dir(&package_dir)
        .arg("init")
        .assert()
        .success();

    // Validate post-conditions.
    config.assert(predicate::path::exists());
    template.assert(predicate::path::exists());

    Ok(())
}

#[test]
fn init_no_handlebars_generates_config_only() -> Result<()> {
    // Create a minimal valid rust package.
    let package_dir = TempDir::new()?;
    package_dir.child("src/main.rs").touch()?;
    let cargo_toml = package_dir.child("Cargo.toml");
    cargo_toml.touch()?;
    let cargo_toml_content = toml::toml! {
        [package]
        name = "package"
        version = "0.0.0"
    };
    cargo_toml.write_str(&cargo_toml_content.to_string())?;
    let config = package_dir.child("about.toml");
    let template = package_dir.child("about.hbs");

    // Validate the preconditions.
    config.assert(predicate::path::missing());
    template.assert(predicate::path::missing());

    // Invoke into to generate the default `about.hbs` template.
    Command::cargo_bin("cargo-about")?
        .current_dir(&package_dir)
        .arg("init")
        .arg("--no-handlebars")
        .assert()
        .success()
        .stdout("")
        .stderr("");

    // Validate post-conditions.
    config.assert(predicate::path::exists());
    template.assert(predicate::path::missing());

    Ok(())
}

#[test]
fn init_does_not_overwrite() -> Result<()> {
    // Create a minimal valid rust package.
    let package_dir = TempDir::new()?;

    package_dir.child("src/main.rs").touch()?;

    let cargo_toml = package_dir.child("Cargo.toml");
    cargo_toml.touch()?;
    let cargo_toml_content = toml::toml! {
        [package]
        name = "package"
        version = "0.0.0"
    };
    cargo_toml.write_str(&cargo_toml_content.to_string())?;

    let template = package_dir.child("about.hbs");
    let template_content = "A useless custom template";
    template.write_str(template_content)?;

    let config = package_dir.child("about.toml");
    let config_content = "A useless invalid config";
    config.write_str(config_content)?;

    // Invoke into to attempt to generate the default `about.hbs` template.
    Command::cargo_bin("cargo-about")?
        .current_dir(&package_dir)
        .arg("init")
        .assert()
        .success()
        .stdout("")
        .stderr("");

    assert_eq!(std::fs::read_to_string(&config)?, config_content);
    assert_eq!(std::fs::read_to_string(&template)?, template_content);

    Ok(())
}

#[test]
fn init_overwrite_overwrites() -> Result<()> {
    // Create a minimal valid rust package.
    let package_dir = TempDir::new()?;
    package_dir.child("src/main.rs").touch()?;
    let cargo_toml = package_dir.child("Cargo.toml");
    cargo_toml.touch()?;
    let cargo_toml_content = toml::toml! {
        [package]
        name = "package"
        version = "0.0.0"
    };
    cargo_toml.write_str(&cargo_toml_content.to_string())?;

    let template = package_dir.child("about.hbs");
    let template_content = "A useless custom template";
    template.write_str(template_content)?;

    let config = package_dir.child("about.toml");
    let config_content = "A useless invalid config";
    config.write_str(config_content)?;

    // Invoke into to attempt to generate the default `about.hbs` template.
    Command::cargo_bin("cargo-about")?
        .current_dir(&package_dir)
        .arg("init")
        .arg("--overwrite")
        .assert()
        .success()
        .stdout("")
        .stderr("");

    // TODO: would be nice to compare with something from src if easy
    assert_ne!(std::fs::read_to_string(&config)?, config_content);
    assert_ne!(std::fs::read_to_string(&template)?, template_content);

    Ok(())
}

#[test]
fn init_no_handlebars_overwrite_overwrites_config_only() -> Result<()> {
    // Create a minimal valid rust package.
    let package_dir = TempDir::new()?;
    package_dir.child("src/main.rs").touch()?;
    let cargo_toml = package_dir.child("Cargo.toml");
    cargo_toml.touch()?;
    let cargo_toml_content = toml::toml! {
        [package]
        name = "package"
        version = "0.0.0"
    };
    cargo_toml.write_str(&cargo_toml_content.to_string())?;

    let template = package_dir.child("about.hbs");
    let template_content = "A useless custom template";
    template.write_str(template_content)?;

    let config = package_dir.child("about.toml");
    let config_content = "A useless invalid config";
    config.write_str(config_content)?;

    // Invoke into to attempt to generate the default `about.hbs` template.
    Command::cargo_bin("cargo-about")?
        .current_dir(&package_dir)
        .arg("init")
        .arg("--no-handlebars")
        .arg("--overwrite")
        .assert()
        .success()
        .stdout("")
        .stderr("");

    // TODO: would be nice to compare with something from src if easy
    assert_ne!(std::fs::read_to_string(&config)?, config_content);
    assert_eq!(std::fs::read_to_string(&template)?, template_content);

    Ok(())
}
