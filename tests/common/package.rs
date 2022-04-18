use anyhow::Result;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use core::fmt::Write;
use indoc::indoc;
use std::collections::HashMap;
use std::collections::HashSet;

const CARGO_MANIFEST_FILENAME: &str = "Cargo.toml";
const ABOUT_CONFIG_FILENAME: &str = "about.toml";
const ABOUT_TEMPLATE_FILENAME: &str = "about.hbs";

pub struct Package {
    pub dir: TempDir,
    pub name: String,
    pub version: String,
    pub template_filename: Option<String>,
}

impl Package {
    pub fn builder<'a>() -> PackageBuilder<'a> {
        PackageBuilder::default()
    }
}

#[allow(dead_code)]
pub fn inspect(package: &Package) -> Result<()> {
    println!(
        "{}",
        std::fs::read_to_string(package.dir.child(CARGO_MANIFEST_FILENAME))?
    );
    println!(
        "{}",
        std::fs::read_to_string(package.dir.child(ABOUT_CONFIG_FILENAME))?
    );
    println!(
        "src/main.rs exists: {}",
        package.dir.child("src/main.rs").exists()
    );
    println!("end - - ---------------------------------------------");
    Ok(())
}

pub struct PackageBuilder<'a> {
    name: String,
    version: String,
    license: Option<String>,
    license_filename: Option<String>,
    accepted: HashSet<String>,
    files: HashMap<String, String>,
    excludes: HashSet<String>,
    dependencies: Vec<&'a Package>,
}

impl Default for PackageBuilder<'_> {
    fn default() -> Self {
        PackageBuilder {
            name: "package".into(),
            version: "0.0.0".into(),
            license: None,
            license_filename: None,
            accepted: HashSet::new(),
            files: HashMap::new(),
            excludes: HashSet::new(),
            dependencies: Vec::new(),
        }
    }
}

impl<'a> PackageBuilder<'a> {
    pub fn no_manifest(&mut self) -> &mut Self {
        self.excludes.insert(CARGO_MANIFEST_FILENAME.into());
        self
    }

    #[allow(dead_code)]
    pub fn no_about_config(&mut self) -> &mut Self {
        self.excludes.insert(ABOUT_CONFIG_FILENAME.into());
        self
    }

    pub fn no_template(&mut self) -> &mut Self {
        self.excludes.insert(ABOUT_TEMPLATE_FILENAME.into());
        self
    }

    pub fn name(&mut self, name: &str) -> &mut Self {
        self.name = name.into();
        self
    }

    #[allow(dead_code)]
    pub fn version(&mut self, version: &str) -> &mut Self {
        self.version = version.into();
        self
    }

    pub fn license(&mut self, license: Option<&str>) -> &mut Self {
        self.license = license.map(|s| s.into());
        self
    }

    pub fn license_file(&mut self, filename: &str, content: Option<&str>) -> &mut Self {
        self.license_filename = Some(filename.into());
        if let Some(content) = content {
            self.files.insert(filename.into(), content.into());
        } else {
            self.files.remove(filename);
        }
        self
    }

    pub fn file(&mut self, filename: &str, content: &str) -> &mut Self {
        self.files.insert(filename.into(), content.into());
        self
    }

    pub fn add_accepted(&mut self, name: &str) -> &mut Self {
        self.accepted.insert(name.into());
        self
    }

    pub fn dependency(&mut self, package: &'a Package) -> &mut Self {
        self.dependencies.push(package);
        self
    }

    pub fn with_mit_license_field_defaults(&mut self) -> &mut Self {
        self.license(Some("MIT"))
            .file("src/main.rs", "")
            .add_accepted("MIT")
    }

    fn not_overridden_or_excluded(&self, filename: &str) -> bool {
        !self.files.contains_key(filename) && !self.excludes.contains(filename)
    }

    fn write_default_cargo_manifest_if_absent(&self, dir: &TempDir) -> Result<()> {
        if self.not_overridden_or_excluded(CARGO_MANIFEST_FILENAME) {
            let mut content = String::new();

            writeln!(&mut content, "[package]")?;
            writeln!(&mut content, "name = \"{}\"", self.name)?;
            writeln!(&mut content, "version = \"{}\"", self.version)?;
            if let Some(license) = &self.license {
                writeln!(&mut content, "license = \"{}\"", license)?;
            }
            if let Some(license_filename) = &self.license_filename {
                writeln!(&mut content, "license_file = \"{}\"", license_filename)?;
            }

            if !self.dependencies.is_empty() {
                writeln!(&mut content)?;
                writeln!(&mut content, "[dependencies]")?;
                for package in &self.dependencies {
                    writeln!(
                        &mut content,
                        "{} = {{ path = \"{}\"  }}",
                        package.name,
                        package.dir.to_str().unwrap()
                    )?;
                }
            }

            dir.child(CARGO_MANIFEST_FILENAME).write_str(&content)?;
        }

        Ok(())
    }

    fn write_default_about_config_if_absent(&self, dir: &TempDir) -> Result<()> {
        if self.not_overridden_or_excluded(ABOUT_CONFIG_FILENAME) {
            let mut content = String::new();

            // accepted field
            write!(&mut content, "accepted = [ ")?;
            if !self.accepted.is_empty() {
                write!(
                    &mut content,
                    "{}",
                    self.accepted
                        .iter()
                        .map(|s| format!("\"{}\"", s))
                        .collect::<Vec<_>>()
                        .join(", ")
                )?;
                write!(&mut content, " ")?;
            }
            writeln!(&mut content, "]")?;

            dir.child(ABOUT_CONFIG_FILENAME).write_str(&content)?;
        }

        Ok(())
    }

    // required for package to contain at least one valid crate
    fn write_default_main_if_absent(&self, dir: &TempDir) -> Result<()> {
        let filename = "src/main.rs";
        if self.not_overridden_or_excluded(filename) {
            dir.child(filename).write_str("")?;
        }

        Ok(())
    }

    fn write_default_template_if_absent(&self, dir: &TempDir) -> Result<()> {
        if self.not_overridden_or_excluded(ABOUT_TEMPLATE_FILENAME) {
            // Getting the number of overview and licenses elements
            // by repeating a single letter on a line. This is a
            // workaround for the fact that there doesn't seem to
            // be a built in helper/property for getting a list's
            // length in the rust implementation of handlebars.
            dir.child(ABOUT_TEMPLATE_FILENAME).write_str(indoc! {r#"
                    #o:[{{#each overview}}o{{/each}}]
                    {{#each overview}}
                    o,{{count}},{{name}},{{id}}
                    {{/each}}

                    #l:[{{#each licenses}}l{{/each}}]
                    {{#each licenses}}
                    l,{{name}},{{id}},{{source_path}},{{{text}}}
                    {{/each}}
                "#})?;
        }

        Ok(())
    }

    pub fn write_files(&self, dir: &TempDir) -> Result<()> {
        for (filename, content) in &self.files {
            if !self.excludes.contains(filename) {
                let file = dir.child(&filename);
                file.write_str(content)?;
            }
        }

        Ok(())
    }

    pub fn build(&self) -> Result<Package> {
        let dir = TempDir::new()?;
        self.write_default_cargo_manifest_if_absent(&dir)?;
        self.write_default_about_config_if_absent(&dir)?;
        self.write_default_main_if_absent(&dir)?;
        self.write_default_template_if_absent(&dir)?;
        self.write_files(&dir)?;

        Ok(Package {
            dir,
            template_filename: if self.excludes.contains(ABOUT_TEMPLATE_FILENAME) {
                None
            } else {
                Some(ABOUT_TEMPLATE_FILENAME.into())
            },
            name: self.name.clone(),
            version: self.version.clone(),
        })
    }
}
