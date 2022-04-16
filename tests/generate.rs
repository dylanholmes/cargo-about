mod common;

use anyhow::Result;
use assert_cmd::prelude::*;
use common::*;
use indoc::indoc;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn generate_fails_when_templates_arg_missing() -> Result<()> {
    About::generate()?
        .assert()
        .failure()
        .stderr(predicate::str::is_match(
            r"required arguments were not provided:\s*<TEMPLATES>",
        )?);

    Ok(())
}

#[test]
fn generate_fails_when_manifest_absent() -> Result<()> {
    About::generate()?
        //.arg("my-about.hbs")
        .template("my-about.hbs", None)?
        .assert()
        .failure()
        .stderr(predicate::str::is_match(
            r"cargo manifest path '.*' does not exist",
        )?);

    Ok(())
}

#[test]
fn generate_fails_when_manifest_invalid() -> Result<()> {
    let package = Package::builder().file("Cargo.toml", "").build()?;

    Command::cargo_bin("cargo-about")?
        .current_dir(&package.dir)
        .arg("generate")
        .arg("my-about.hbs")
        .assert()
        .failure()
        .stderr(predicate::str::contains("failed to parse manifest"));

    Ok(())
}

#[test]
fn generate_fails_when_template_file_missing() -> Result<()> {
    let package = Package::builder().dummy_main().build()?;

    Command::cargo_bin("cargo-about")?
        .current_dir(&package.dir)
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
    let package = Package::builder()
        .dummy_main()
        .with_simple_template()
        .build()?;

    Command::cargo_bin("cargo-about")?
        .current_dir(&package.dir)
        .arg("generate")
        .arg("my-about.hbs")
        .assert()
        .success()
        .stdout("\n");

    Ok(())
}

#[test]
fn generate_fails_when_missing_accepted_field() -> Result<()> {
    let package = Package::builder()
        .dummy_main()
        .with_simple_template()
        .file("about.toml", "")
        .build()?;

    Command::cargo_bin("cargo-about")?
        .current_dir(&package.dir)
        .arg("generate")
        .arg("my-about.hbs")
        .assert()
        .failure()
        .stderr(predicates::str::contains("missing field `accepted`"));

    Ok(())
}

#[test]
fn generate_succeeds_with_warning_when_no_license_and_accepted_field_empty() -> Result<()> {
    let package = Package::builder()
        .dummy_main()
        .template_file("my-about.hbs", Some(""))
        .build()?;

    Command::cargo_bin("cargo-about")?
        .current_dir(&package.dir)
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
    let package = Package::builder()
        .license(Some("MIT"))
        .template_file("my-about.hbs", Some(""))
        .dummy_main()
        .build()?;

    Command::cargo_bin("cargo-about")?
        .current_dir(&package.dir)
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
    let package = Package::builder()
        .with_mit_license_field_defaults()
        .license(Some("UNKNOWN"))
        .build()?;

    Command::cargo_bin("cargo-about")?
        .current_dir(&package.dir)
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
fn generate_succeeds_when_license_field_valid() -> Result<()> {
    let package = Package::builder()
        .with_mit_license_field_defaults()
        .build()?;

    let contains_mit_overview = predicates::str::contains("o,1,MIT License,MIT");
    let contains_mit_license = predicates::str::contains("l,MIT License,MIT,");

    Command::cargo_bin("cargo-about")?
        .current_dir(&package.dir)
        .arg("generate")
        .arg("my-about.hbs")
        .assert()
        .success()
        .stdout(contains_mit_overview)
        .stdout(contains_mit_license);

    Ok(())
}

#[test]
fn generate_succeeds_with_warning_when_license_file_field_but_no_file() -> Result<()> {
    let package = Package::builder()
        .dummy_main()
        .with_simple_template()
        .license_file("MIT_LICENSE", None)
        .add_accepted("MIT")
        .build()?;

    Command::cargo_bin("cargo-about")?
        .current_dir(&package.dir)
        .arg("generate")
        .arg("my-about.hbs")
        .assert()
        .success()
        // TODO: might be nice to let the user know that there was a license file field, but
        // that the file was missing.
        .stderr(predicates::str::contains(
            "unable to synthesize license expression for 'package 0.0.0': \
            no `license` specified, and no license files were found",
        ))
        .stdout("\n");

    Ok(())
}

#[test]
fn generate_succeeds_with_warning_when_license_file_field_but_file_empty() -> Result<()> {
    let package = Package::builder()
        .dummy_main()
        .with_simple_template()
        .license_file("MIT_LICENSE", Some(""))
        .add_accepted("MIT")
        .build()?;

    Command::cargo_bin("cargo-about")?
        .current_dir(&package.dir)
        .arg("generate")
        .arg("my-about.hbs")
        .assert()
        .success()
        // TODO: might be nice to let the user know that there was a license file field, but
        // that the file was missing.
        .stderr(predicates::str::contains(
            "unable to synthesize license expression for 'package 0.0.0': \
            no `license` specified, and no license files were found",
        ))
        .stdout("\n");

    Ok(())
}

// TODO: This seems like incorrect behavior.... IMO the report should be generated
// and maybe custom and/or non-accepted licenses should be included with some
// additional metadata noting that it is not accepted..
#[test]
fn generate_succeeds_with_warning_when_non_spdx_license_file() -> Result<()> {
    let package = Package::builder()
        .dummy_main()
        .with_simple_template()
        .license_file(
            "LICENSE",
            Some("Copyright © 2022 Big Birdz. No permissions granted ever."),
        )
        .build()?;

    Command::cargo_bin("cargo-about")?
        .current_dir(&package.dir)
        .arg("generate")
        .arg("my-about.hbs")
        .assert()
        .success()
        // TODO: might be nice to let the user know that there was a license file field, but
        // that the file was missing.
        .stderr(predicates::str::contains(
            "unable to synthesize license expression for 'package 0.0.0': \
            no `license` specified, and no license files were found",
        ))
        .stdout("\n");

    Ok(())
}

// TODO: This seems like incorrect behavior.... IMO the report should be generated
// and maybe custom and/or non-accepted licenses should be included with some
// additional metadata noting that it is not accepted..
#[test]
fn generate_succeeds_with_warning_when_spdx_license_file() -> Result<()> {
    let package = Package::builder()
        .dummy_main()
        .with_simple_template()
        .license_file("LICENSE", Some(&common::mit_license_content("Big Birdz")))
        .add_accepted("MIT")
        .build()?;

    let contains_mit_overview = predicates::str::contains("o,1,MIT License,MIT");
    let contains_mit_license = predicates::str::contains("l,MIT License,MIT,");

    Command::cargo_bin("cargo-about")?
        .current_dir(&package.dir)
        .arg("generate")
        .arg("my-about.hbs")
        .assert()
        .success()
        // TODO: might be nice to let the user know that there was a license file field, but
        // that the file was missing.
        // TODO: This should not be a warning, since the crate does have a license file field.
        .stderr(predicates::str::contains(
            "crate 'package 0.0.0' doesn't have a license field",
        ))
        .stdout(contains_mit_overview)
        .stdout(contains_mit_license);

    Ok(())
}

#[test]
fn generate_succeeds_with_warning_when_spdx_license_file_non_std_naming() -> Result<()> {
    let package = Package::builder()
        .dummy_main()
        .with_simple_template()
        .license_file(
            "MIT_LICENSE",
            Some(&common::mit_license_content("Big Birdz")),
        )
        .add_accepted("MIT")
        .build()?;

    Command::cargo_bin("cargo-about")?
        .current_dir(&package.dir)
        .arg("generate")
        .arg("my-about.hbs")
        .assert()
        .success()
        .stderr(predicates::str::contains(
            "unable to synthesize license expression for 'package 0.0.0': \
            no `license` specified, and no license files were found",
        ))
        // TODO: This seems like a bug. I would've expected this to detect
        // the MIT license just the same as when the license file is named
        // "LICENSE", but it doesn't.
        .stdout("\n");

    Ok(())
}

// TODO: If a license file is given and an spdx identifier exists,
// then the cutom file should be used.
#[test]
fn generate_succeeds_when_custom_spdx_license_file() -> Result<()> {
    let package = Package::builder()
        .dummy_main()
        .with_simple_template()
        .license(Some("MIT"))
        .add_accepted("MIT")
        .file("LICENSE", &mit_license_content("Big Birdz"))
        .build()?;

    let contains_mit_overview = predicates::str::contains("o,1,MIT License,MIT");
    let contains_mit_license = predicates::str::contains("l,MIT License,MIT,");
    let contains_mit_license_text = predicates::str::contains(&mit_license_content("Big Birdz"));

    Command::cargo_bin("cargo-about")?
        .current_dir(&package.dir)
        .arg("generate")
        .arg("my-about.hbs")
        .assert()
        .success()
        // TODO: might be nice to let the user know that there was a license file field, but
        // that the file was missing.
        //.stderr(predicates::str::contains(
        //    "unable to synthesize license expression for 'package 0.0.0': \
        //    no `license` specified, and no license files were found",
        //))
        .stdout(contains_mit_overview)
        .stdout(contains_mit_license)
        .stdout(contains_mit_license_text);

    Ok(())
}

// Out of Scope
// - testing all SPDX Identifiers, that should be handled by the spdx crate which uses data from
// - testing all SPDX expressions
// Single Package -- License Field -- All SPDX Licenses are generated and any custom license file
// is used

// Single Package -- License File -- All SPDX Licenses are recovered and custom license file is
// used
