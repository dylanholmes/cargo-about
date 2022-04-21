mod package;
mod predicates;

pub use self::package::*;
pub use self::predicates::*;

use anyhow::Result;
use assert_cmd::assert::Assert;
use assert_cmd::prelude::*;
use std::env::current_dir;
use std::process::Command;

pub struct About {
    cmd: Command,
}

impl About {
    pub fn new(package: &Package) -> Result<Self> {
        let mut cmd = Command::cargo_bin("cargo-about")?;
        if std::env::var("GENERATE_LLVM_PROFILE_FILES").is_ok() {
            cmd.env(
                "LLVM_PROFILE_FILE",
                format!(
                    "{}/cargo-about-generate-tests%c.profraw",
                    current_dir()?.as_path().to_str().unwrap()
                ),
            );
        }
        cmd.current_dir(&package.dir);
        Ok(About { cmd })
    }
    pub fn arg(&mut self, arg: &str) -> &mut Self {
        self.cmd.arg(arg);
        self
    }
    pub fn init(&mut self) -> &mut Self {
        self.arg("init")
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
