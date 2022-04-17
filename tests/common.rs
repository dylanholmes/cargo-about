use anyhow::Result;
use assert_cmd::assert::Assert;
use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use core::fmt::Write;
use indoc::formatdoc;
use indoc::indoc;
use std::collections::HashMap;
use std::collections::HashSet;
use std::process::Command;

pub struct Package {
    pub dir: TempDir,
    name: String,
    version: String,
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
        std::fs::read_to_string(package.dir.child("Cargo.toml"))?
    );
    println!(
        "{}",
        std::fs::read_to_string(package.dir.child("about.toml"))?
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
    template_filename: Option<String>,
    files: HashMap<String, String>,
    dependencies: Vec<&'a Package>,
}

impl<'a> PackageBuilder<'a> {
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
    pub fn template_file(&mut self, filename: &str, content: Option<&str>) -> &mut Self {
        self.template_filename = Some(filename.into());
        if let Some(content) = content {
            self.files.insert(filename.into(), content.into());
        } else {
            self.files.remove(filename);
        }
        self
    }
    pub fn dependency(&mut self, package: &'a Package) -> &mut Self {
        self.dependencies.push(package);
        self
    }

    pub fn dummy_main(&mut self) -> &mut Self {
        self.file("src/main.rs", "// dummy main")
    }

    pub fn with_simple_template(&mut self) -> &mut Self {
        // Getting the number of overview and licenses elements
        // by repeating a single letter on a line. This is a
        // workaround for the fact that there doesn't seem to
        // be a built in helper/property for getting a list's
        // length in the rust implementation of handlebars.
        self.template_file(
            "my-about.hbs",
            Some(indoc! {r#"
                    #o:[{{#each overview}}o{{/each}}]
                    {{#each overview}}
                    o,{{count}},{{name}},{{id}}
                    {{/each}}

                    #l:[{{#each licenses}}l{{/each}}]
                    {{#each licenses}}
                    l,{{name}},{{id}},{{source_path}},{{{text}}}
                    {{/each}}
                "#}),
        )
    }

    pub fn with_mit_license_field_defaults(&mut self) -> &mut Self {
        self.with_simple_template()
            .license(Some("MIT"))
            .file("src/main.rs", "")
            .add_accepted("MIT")
    }

    fn build_cargo_manifest(&self, dir: &TempDir) -> Result<()> {
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

        dir.child("Cargo.toml").write_str(&content)?;

        Ok(())
    }

    fn build_about_config(&self, dir: &TempDir) -> Result<()> {
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

        dir.child("about.toml").write_str(&content)?;

        Ok(())
    }

    pub fn build_files(&self, dir: &TempDir) -> Result<()> {
        for (filename, content) in &self.files {
            let file = dir.child(&filename);
            file.write_str(content)?;
        }

        Ok(())
    }

    pub fn build(&self) -> Result<Package> {
        let dir = TempDir::new()?;

        self.build_cargo_manifest(&dir)?;
        self.build_about_config(&dir)?;
        self.build_files(&dir)?;

        Ok(Package {
            dir,
            name: self.name.clone(),
            version: self.version.clone(),
        })
    }
}

impl Default for PackageBuilder<'_> {
    fn default() -> Self {
        PackageBuilder {
            name: "package".into(),
            version: "0.0.0".into(),
            license: None,
            license_filename: None,
            accepted: HashSet::new(),
            template_filename: None,
            files: HashMap::new(),
            dependencies: Vec::new(),
        }
    }
}

pub struct About {
    cmd: Command,
    current_dir: TempDir,
}

impl About {
    fn new() -> Result<Self> {
        let mut about = Self {
            cmd: Command::cargo_bin("cargo-about")?,
            current_dir: TempDir::new()?,
        };
        about.cmd.current_dir(&about.current_dir);

        Ok(about)
    }
    pub fn generate() -> Result<AboutGenerate> {
        let mut about = Self::new()?;
        about.cmd.arg("generate");
        Ok(AboutGenerate { about })
    }
}

pub struct AboutGenerate {
    about: About,
}

impl AboutGenerate {
    pub fn template(&mut self, filename: &str, contents: Option<&str>) -> Result<&mut Self> {
        self.about.cmd.arg(filename);
        if let Some(contents) = contents {
            self.about.current_dir.child(filename).write_str(contents)?;
        }
        Ok(self)
    }
    pub fn assert(&mut self) -> Assert {
        self.about.cmd.assert()
    }
}

pub fn mit_license_content(year: &str, copyright_holder: &str) -> String {
    formatdoc! {r#"
            Copyright (c) {year} {copyright_holder}

            Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
            
            The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
            
            THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
    "#,
    copyright_holder = copyright_holder
    }
}

pub fn contains_default_mit_license_content() -> predicates::str::ContainsPredicate {
    contains_mit_license_content("<year>", "<copyright holders>")
}

pub fn contains_mit_license_content(
    year: &str,
    copyright_holder: &str,
) -> predicates::str::ContainsPredicate {
    predicates::str::contains(mit_license_content(year, copyright_holder))
}

pub fn overview_count(count: usize) -> predicates::str::ContainsPredicate {
    predicates::str::contains(format!("#o:[{}]", "o".repeat(count)))
}

pub fn licenses_count(count: usize) -> predicates::str::ContainsPredicate {
    predicates::str::contains(format!("#l:[{}]", "l".repeat(count)))
}

pub fn no_licenses_found(package: &Package) -> predicates::str::ContainsPredicate {
    predicates::str::contains(format!(
        "unable to synthesize license expression for '{} {}': \
            no `license` specified, and no license files were found",
        package.name, package.version
    ))
}
