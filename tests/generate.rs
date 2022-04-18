mod common;

use anyhow::Result;
use common::*;
use predicates::prelude::*;

#[test]
fn generate_fails_when_templates_arg_missing() -> Result<()> {
    let package = Package::builder().no_template().build()?;

    About::generate(&package)?
        .assert()
        .failure()
        .stderr(predicate::str::is_match(
            r"required arguments were not provided:\s*<TEMPLATES>",
        )?);

    Ok(())
}

#[test]
fn generate_fails_when_manifest_absent() -> Result<()> {
    let package = Package::builder().no_manifest().build()?;

    About::generate(&package)?
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

    About::generate(&package)?
        .assert()
        .failure()
        .stderr(predicate::str::contains("failed to parse manifest"));

    Ok(())
}

#[test]
fn generate_fails_when_template_file_missing() -> Result<()> {
    let package = Package::builder().no_template().build()?;

    About::generate(&package)?
        .arg("non-existent-about.hbs")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "template(s) path non-existent-about.hbs does not exist",
        ));

    Ok(())
}

#[test]
fn generate_succeeds_with_warning_when_no_licenses() -> Result<()> {
    let package = Package::builder().build()?;

    About::generate(&package)?
        .assert()
        .success()
        .stderr(no_licenses_found(&package))
        .stdout(overview_count(0))
        .stdout(licenses_count(0));

    Ok(())
}

#[test]
fn generate_fails_when_missing_accepted_field() -> Result<()> {
    let package = Package::builder().file("about.toml", "").build()?;

    About::generate(&package)?
        .assert()
        .failure()
        .stderr(predicates::str::contains("missing field `accepted`"));

    Ok(())
}

#[test]
fn generate_succeeds_with_warning_when_no_license_and_accepted_field_empty() -> Result<()> {
    let package = Package::builder().build()?;

    About::generate(&package)?
        .assert()
        .success()
        .stderr(no_licenses_found(&package))
        .stdout(overview_count(0))
        .stdout(licenses_count(0));

    Ok(())
}

#[test]
fn generate_fails_when_license_field_valid_and_accepted_field_empty() -> Result<()> {
    let package = Package::builder().license(Some("MIT")).build()?;

    About::generate(&package)?
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

    About::generate(&package)?
        .assert()
        .success()
        .stderr(predicates::str::contains(
            "unable to parse license expression for 'package 0.0.0': UNKNOWN",
        ))
        .stdout(overview_count(0))
        .stdout(licenses_count(0));

    Ok(())
}

#[test]
fn generate_succeeds_when_license_field_valid() -> Result<()> {
    let package = Package::builder()
        .with_mit_license_field_defaults()
        .build()?;

    let contains_mit_overview = predicates::str::contains("o,1,MIT License,MIT");
    let contains_mit_license = predicates::str::contains("l,MIT License,MIT,");

    About::generate(&package)?
        .assert()
        .success()
        .stdout(contains_mit_overview)
        .stdout(contains_mit_license);

    Ok(())
}

#[test]
fn generate_succeeds_with_warning_when_license_file_field_but_no_file() -> Result<()> {
    let package = Package::builder()
        .license_file("MIT_LICENSE", None)
        .add_accepted("MIT")
        .build()?;

    About::generate(&package)?
        .assert()
        .success()
        // TODO: might be nice to let the user know that there was a license file field, but
        // that the file was missing.
        .stderr(no_licenses_found(&package))
        .stdout(overview_count(0))
        .stdout(licenses_count(0));

    Ok(())
}

#[test]
fn generate_succeeds_with_warning_when_license_file_field_but_file_empty() -> Result<()> {
    let package = Package::builder()
        .license_file("MIT_LICENSE", Some(""))
        .add_accepted("MIT")
        .build()?;

    About::generate(&package)?
        .assert()
        .success()
        // TODO: might be nice to let the user know that there was a license file field, but
        // that the file was missing.
        .stderr(no_licenses_found(&package))
        .stdout(overview_count(0))
        .stdout(licenses_count(0));

    Ok(())
}

// TODO: This seems like incorrect behavior.... IMO the report should be generated
// and maybe custom and/or non-accepted licenses should be included with some
// additional metadata noting that it is not accepted..
#[test]
fn generate_succeeds_with_warning_when_non_spdx_license_file() -> Result<()> {
    let package = Package::builder()
        .license_file(
            "LICENSE",
            Some("Copyright © 2022 Big Birdz. No permissions granted ever."),
        )
        .build()?;

    About::generate(&package)?
        .assert()
        .success()
        // TODO: might be nice to let the user know that there was a license file field, but
        // that the file was missing.
        .stderr(no_licenses_found(&package))
        .stdout(overview_count(0))
        .stdout(licenses_count(0));

    Ok(())
}

// TODO: This seems like incorrect behavior.... IMO the report should be generated
// and maybe custom and/or non-accepted licenses should be included with some
// additional metadata noting that it is not accepted..
#[test]
fn generate_succeeds_with_warning_when_spdx_license_file() -> Result<()> {
    let package = Package::builder()
        .license_file(
            "LICENSE",
            Some(&common::mit_license_content("2022", "Big Birdz")),
        )
        .add_accepted("MIT")
        .build()?;

    let contains_mit_overview = predicates::str::contains("o,1,MIT License,MIT");
    let contains_mit_license = predicates::str::contains("l,MIT License,MIT,");

    About::generate(&package)?
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

// TODO: This seems like a bug. I would've expected this to detect
// the MIT license just the same as when the license file is named
// "LICENSE", but it doesn't.
#[test]
fn generate_succeeds_with_warning_when_spdx_license_file_non_std_naming() -> Result<()> {
    let package = Package::builder()
        .license_file(
            "MIT_LICENSE",
            Some(&common::mit_license_content("2022", "Big Birdz")),
        )
        .add_accepted("MIT")
        .build()?;

    About::generate(&package)?
        .assert()
        .success()
        .stderr(no_licenses_found(&package))
        .stdout(overview_count(0))
        .stdout(licenses_count(0));

    Ok(())
}

#[test]
fn generate_succeeds_when_custom_spdx_license_file() -> Result<()> {
    let package = Package::builder()
        .name("package")
        .license(Some("MIT"))
        .add_accepted("MIT")
        .file("LICENSE", &mit_license_content("2022", "Big Birdz"))
        .build()?;

    let contains_mit_overview = predicates::str::contains("o,1,MIT License,MIT");
    let contains_mit_license = predicates::str::contains("l,MIT License,MIT,");
    let contains_mit_license_text =
        predicates::str::contains(&mit_license_content("2022", "Big Birdz"));

    About::generate(&package)?
        .assert()
        .success()
        .stderr("")
        .stdout(overview_count(1))
        .stdout(licenses_count(1))
        .stdout(contains_mit_overview)
        .stdout(contains_mit_license)
        .stdout(contains_mit_license_text);

    Ok(())
}

#[test]
fn generate_succeeds_when_dependency_has_spdx_license_field() -> Result<()> {
    let mut package_builder = Package::builder();
    package_builder.license(Some("MIT"));

    let package_b = package_builder.name("package-b").build()?;
    let package_a = package_builder
        .name("package-a")
        .add_accepted("MIT")
        .dependency(&package_b)
        .build()?;

    let contains_mit_overview = predicates::str::contains("o,1,MIT License,MIT");
    let contains_mit_license = predicates::str::contains("l,MIT License,MIT,");

    About::generate(&package_a)?
        .assert()
        .success()
        .stderr("")
        .stdout(overview_count(1))
        .stdout(licenses_count(1))
        .stdout(contains_mit_overview)
        .stdout(contains_mit_license)
        .stdout(contains_default_mit_license_content());

    Ok(())
}

#[test]
fn generate_fails_when_dependency_has_no_spdx_license_field() -> Result<()> {
    let mut package_builder = Package::builder();

    let package_b = package_builder
        .license(Some("Apache-2.0"))
        .name("package-b")
        .build()?;

    let package_a = package_builder
        .license(Some("MIT"))
        .name("package-a")
        .add_accepted("MIT")
        .dependency(&package_b)
        .build()?;

    About::generate(&package_a)?.assert().failure();

    Ok(())
}

// Out of Scope
// - testing all SPDX Identifiers, that should be handled by the spdx crate which uses data from
// - testing all SPDX expressions
// Single Package -- License Field -- All SPDX Licenses are generated and any custom license file
// is used

// Single Package -- License File -- All SPDX Licenses are recovered and custom license file is
// used
