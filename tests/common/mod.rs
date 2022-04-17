mod package;
mod predicates;

pub use crate::common::package::*;
pub use crate::common::predicates::*;

use anyhow::Result;
use assert_cmd::assert::Assert;
use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use std::process::Command;

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
