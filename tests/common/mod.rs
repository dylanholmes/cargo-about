mod package;
mod predicates;

pub use crate::common::package::*;
pub use crate::common::predicates::*;

use anyhow::Result;
use assert_cmd::assert::Assert;
use assert_cmd::prelude::*;
use std::process::Command;

pub struct About {
    cmd: Command,
}

impl About {
    pub fn new(package: &Package) -> Result<Self> {
        let mut cmd = Command::cargo_bin("cargo-about")?;
        cmd.current_dir(&package.dir);
        Ok(About { cmd })
    }
    pub fn arg(&mut self, arg: &str) -> &mut Self {
        self.cmd.arg(arg);
        self
    }
    pub fn generate(&mut self) -> &mut Self {
        self.arg("generate")
    }
    pub fn template(&mut self, template: &str) -> &mut Self {
        self.arg(template)
    }
    pub fn assert(&mut self) -> Assert {
        self.cmd.assert()
    }
}
