use crate::utils::*;

use anyhow::Result;
use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
fn init_reports_error_when_manifest_absent() -> Result<()> {
    let package = Package::builder().no_manifest().build()?;

    About::new(&package)?
        .init()
        .assert()
        .failure()
        .stderr(predicate::str::contains("could not find `Cargo.toml`"));
    Ok(())
}

#[test]
fn init_reports_error_when_manifest_empty() -> Result<()> {
    let package = Package::builder().file("Cargo.toml", "").build()?;

    About::new(&package)?
        .init()
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
    let package = Package::builder().no_template().no_about_config().build()?;

    About::new(&package)?.init().assert().success();

    let dir = &package.dir;
    dir.child(ABOUT_CONFIG_FILENAME)
        .assert(predicate::path::exists());
    dir.child(ABOUT_TEMPLATE_FILENAME)
        .assert(predicate::path::exists());

    Ok(())
}

#[test]
fn init_no_handlebars_generates_config_only() -> Result<()> {
    let package = Package::builder().no_template().no_about_config().build()?;

    About::new(&package)?
        .init()
        .arg("--no-handlebars")
        .assert()
        .success()
        .stdout("")
        .stderr("");

    let dir = &package.dir;
    dir.child(ABOUT_CONFIG_FILENAME)
        .assert(predicate::path::exists());
    dir.child(ABOUT_TEMPLATE_FILENAME)
        .assert(predicate::path::missing());

    Ok(())
}

#[test]
fn init_does_not_overwrite() -> Result<()> {
    let template_content = "A useless custom template";
    let config_content = "A useless invalid config";

    let package = Package::builder()
        .file(ABOUT_TEMPLATE_FILENAME, template_content)
        .file(ABOUT_CONFIG_FILENAME, config_content)
        .build()?;

    About::new(&package)?
        .init()
        .assert()
        .success()
        .stdout("")
        .stderr("");

    let config = &package.dir.child(ABOUT_CONFIG_FILENAME);
    let template = &package.dir.child(ABOUT_TEMPLATE_FILENAME);

    assert_eq!(std::fs::read_to_string(&config)?, config_content);
    assert_eq!(std::fs::read_to_string(&template)?, template_content);

    Ok(())
}

#[test]
fn init_overwrite_overwrites() -> Result<()> {
    let template_content = "A useless custom template";
    let config_content = "A useless invalid config";

    let package = Package::builder()
        .file(ABOUT_TEMPLATE_FILENAME, template_content)
        .file(ABOUT_CONFIG_FILENAME, config_content)
        .build()?;

    About::new(&package)?
        .init()
        .arg("--overwrite")
        .assert()
        .success()
        .stdout("")
        .stderr("");

    let config = &package.dir.child(ABOUT_CONFIG_FILENAME);
    let template = &package.dir.child(ABOUT_TEMPLATE_FILENAME);

    assert_ne!(std::fs::read_to_string(&config)?, config_content);
    assert_ne!(std::fs::read_to_string(&template)?, template_content);

    Ok(())
}

#[test]
fn init_no_handlebars_overwrite_overwrites_config_only() -> Result<()> {
    let template_content = "A useless custom template";
    let config_content = "A useless invalid config";

    let package = Package::builder()
        .file(ABOUT_TEMPLATE_FILENAME, template_content)
        .file(ABOUT_CONFIG_FILENAME, config_content)
        .build()?;

    About::new(&package)?
        .init()
        .arg("--no-handlebars")
        .arg("--overwrite")
        .assert()
        .success()
        .stdout("")
        .stderr("");

    let config = &package.dir.child(ABOUT_CONFIG_FILENAME);
    let template = &package.dir.child(ABOUT_TEMPLATE_FILENAME);

    assert_ne!(std::fs::read_to_string(&config)?, config_content);
    assert_eq!(std::fs::read_to_string(&template)?, template_content);

    Ok(())
}
