use crate::utils::*;

use anyhow::Result;
use predicates::prelude::*;

#[test]
fn generate_fails_when_templates_arg_missing() -> Result<()> {
    let package = Package::builder().build()?;

    About::new(&package)?
        .generate()
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

    About::new(&package)?
        .generate()
        .template(&package.template_filename.unwrap())
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

    About::new(&package)?
        .generate()
        .template(&package.template_filename.unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("failed to parse manifest"));

    Ok(())
}

#[test]
fn generate_falls_back_to_default_about_config_when_absent() -> Result<()> {
    let package = Package::builder().no_about_config().build()?;

    About::new(&package)?
        .generate()
        .template(&package.template_filename.unwrap())
        .assert()
        .stderr(predicate::str::contains(
            "no 'about.toml' found, falling back to default configuration",
        ));

    Ok(())
}

#[test]
fn generate_fails_when_template_file_missing() -> Result<()> {
    let package = Package::builder().no_template().build()?;

    About::new(&package)?
        .generate()
        .template("non-existent-about.hbs")
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

    About::new(&package)?
        .generate()
        .template(&package.template_filename.unwrap())
        .assert()
        .success()
        //.stderr(no_licenses_found(&package))
        .stdout(overview_count(0))
        .stdout(licenses_count(0));

    Ok(())
}

#[test]
fn generate_fails_when_missing_accepted_field() -> Result<()> {
    let package = Package::builder().file("about.toml", "").build()?;

    About::new(&package)?
        .generate()
        .template(&package.template_filename.unwrap())
        .assert()
        .failure()
        .stderr(predicates::str::contains("missing field `accepted`"));

    Ok(())
}

#[test]
fn generate_succeeds_with_warning_when_no_license_and_accepted_field_empty() -> Result<()> {
    let package = Package::builder().build()?;

    About::new(&package)?
        .generate()
        .template(&package.template_filename.unwrap())
        .assert()
        .success()
        //.stderr(no_licenses_found(&package))
        .stdout(overview_count(0))
        .stdout(licenses_count(0));

    Ok(())
}

#[test]
fn generate_fails_when_license_field_valid_and_accepted_field_empty() -> Result<()> {
    let package = Package::builder().license(Some("MIT")).build()?;

    About::new(&package)?
        .generate()
        .template(&package.template_filename.unwrap())
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

    About::new(&package)?
        .generate()
        .template(&package.template_filename.unwrap())
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

    About::new(&package)?
        .generate()
        .template(&package.template_filename.unwrap())
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

    About::new(&package)?
        .generate()
        .template(&package.template_filename.unwrap())
        .assert()
        .success()
        // TODO: might be nice to let the user know that there was a license file field, but
        // that the file was missing.
        //.stderr(no_licenses_found(&package))
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

    About::new(&package)?
        .generate()
        .template(&package.template_filename.unwrap())
        .assert()
        .success()
        // TODO: might be nice to let the user know that there was a license file field, but
        // that the file was missing.
        //.stderr(no_licenses_found(&package))
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

    About::new(&package)?
        .generate()
        .template(&package.template_filename.unwrap())
        .assert()
        .success()
        // TODO: might be nice to let the user know that there was a license file field, but
        // that the file was missing.
        //.stderr(no_licenses_found(&package))
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
        .license_file("LICENSE", Some(&mit_license_content("2022", "Big Birdz")))
        .add_accepted("MIT")
        .build()?;

    let contains_mit_overview = predicates::str::contains("o,1,MIT License,MIT");
    let contains_mit_license = predicates::str::contains("l,MIT License,MIT,");

    About::new(&package)?
        .generate()
        .template(&package.template_filename.unwrap())
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
            Some(&mit_license_content("2022", "Big Birdz")),
        )
        .add_accepted("MIT")
        .build()?;

    About::new(&package)?
        .generate()
        .template(package.template_filename.as_ref().unwrap())
        .assert()
        .success()
        .stderr(no_licenses_found(&package))
        .stdout(overview_count(0))
        .stdout(licenses_count(0));

    Ok(())
}

#[test]
#[ignore]
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

    About::new(&package)?
        .generate()
        .template(&package.template_filename.unwrap())
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

    println!("{:?}", package_a);

    About::new(&package_a)?
        .generate()
        .template(&package_a.template_filename.unwrap())
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
fn generate_reports_all_license_texts() -> Result<()> {
    let package_b_license_text = mit_license_content("2022", "Package B Owner");
    let package_b = Package::builder()
        .license_file("LICENSE", Some(&package_b_license_text))
        .name("package-b")
        .build()?;

    let package_a_license_text = mit_license_content("2022", "Package A Owner");
    let package_a = Package::builder()
        .name("package-a")
        .license_file("LICENSE", Some(&package_a_license_text))
        .dependency(&package_b)
        .add_accepted("MIT")
        .build()?;

    About::new(&package_a)?
        .generate()
        .template(&package_a.template_filename.unwrap())
        .assert()
        .success()
        .stdout(overview_count(1))
        .stdout(licenses_count(2))
        .stdout(predicates::str::contains(package_a_license_text))
        .stdout(predicates::str::contains(package_b_license_text));

    Ok(())
}

#[test]
fn generate_succeeds_when_dep_has_different_accepted_license() -> Result<()> {
    let package_b = Package::builder()
        .name("package-b")
        .license(Some("Apache-2.0"))
        .build()?;

    let package_a = Package::builder()
        .name("package-a")
        .license(Some("MIT"))
        .dependency(&package_b)
        .accepted(&["MIT", "Apache-2.0"])
        .build()?;

    About::new(&package_a)?
        .generate()
        .template(&package_a.template_filename.unwrap())
        .assert()
        .success()
        .stdout(overview_count(2))
        .stdout(licenses_count(2));

    Ok(())
}

#[test]
fn generate_fails_when_dependency_has_non_accepted_license_field() -> Result<()> {
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

    About::new(&package_a)?
        .generate()
        .template(&package_a.template_filename.unwrap())
        .assert()
        .failure();

    Ok(())
}

// Out of Scope
// - testing all SPDX Identifiers, that should be handled by the spdx crate which uses data from
// - testing all SPDX expressions
// Single Package -- License Field -- All SPDX Licenses are generated and any custom license file
// is used

// Single Package -- License File -- All SPDX Licenses are recovered and custom license file is
// used
