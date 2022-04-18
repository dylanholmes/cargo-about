mod package;
mod predicates;

pub use crate::common::package::*;
pub use crate::common::predicates::*;

use anyhow::Result;
use assert_cmd::assert::Assert;
use assert_cmd::prelude::*;
use std::process::Command;

pub struct About {}

impl About {
    pub fn generate(package: &Package) -> Result<AboutGenerate> {
        let mut cmd = Command::cargo_bin("cargo-about")?;
        cmd.current_dir(&package.dir);
        cmd.arg("generate");
        if let Some(template) = &package.template_filename {
            cmd.arg(template);
        }
        Ok(AboutGenerate { cmd })
    }
}

pub struct AboutGenerate {
    cmd: Command,
}

impl AboutGenerate {
    pub fn arg(&mut self, arg: &str) -> &mut Self {
        self.cmd.arg(arg);
        self
    }
    pub fn assert(&mut self) -> Assert {
        self.cmd.assert()
    }
}
