use crate::error::{GarnixError, Result};
use serde_json::Value;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

pub struct NixFlake {
    pub flake_path: String,
}

impl NixFlake {
    pub fn new<P: AsRef<Path>>(flake_path: P) -> Result<Self> {
        let flake_path = flake_path.as_ref();

        if !flake_path.join("flake.nix").exists() {
            return Err(GarnixError::NoFlakeFound);
        }

        Ok(Self {
            flake_path: flake_path.to_string_lossy().to_string(),
        })
    }

    pub fn from_git_root<P: AsRef<Path>>(git_root: P) -> Result<Self> {
        Self::new(git_root)
    }

    pub async fn discover_attributes(&self) -> Result<Vec<String>> {
        let output = Command::new("nix")
            .args(["flake", "show", "--json", &self.flake_path])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GarnixError::NixCommand(format!(
                "nix flake show failed: {}",
                stderr
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: Value = serde_json::from_str(&stdout)?;

        let mut attributes = Vec::new();
        let current_system = self.get_current_system().await?;
        self.extract_attributes(&json, Vec::new(), &mut attributes, &current_system);

        Ok(attributes)
    }

    async fn get_current_system(&self) -> Result<String> {
        let output = Command::new("nix")
            .args(["eval", "--expr", "builtins.currentSystem", "--impure"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            return Ok("x86_64-linux".to_string());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(self.clean_nix_output(&stdout))
    }

    fn clean_nix_output(&self, output: &str) -> String {
        output.trim().trim_matches('"').to_string()
    }

    fn extract_attributes(
        &self,
        value: &Value,
        path: Vec<String>,
        attributes: &mut Vec<String>,
        current_system: &str,
    ) {
        if let Value::Object(map) = value {
            for (key, value) in map {
                let mut new_path = path.clone();
                new_path.push(key.clone());

                if path.len() == 1
                    && !key.is_empty()
                    && key != current_system
                    && key.contains("-")
                    && (key.starts_with("x86_64-") || key.starts_with("aarch64-"))
                {
                    continue;
                }

                let is_special_buildable = matches!(
                    path.first().map(|p| p.as_str()),
                    Some("darwinConfigurations" | "homeConfigurations")
                );

                if self.is_buildable_attribute(value) || is_special_buildable {
                    attributes.push(new_path.join("."));
                } else {
                    self.extract_attributes(value, new_path, attributes, current_system);
                }
            }
        }
    }

    fn is_buildable_attribute(&self, value: &Value) -> bool {
        match value {
            Value::Object(map) => match map.get("type").map(|t| t.as_str()) {
                Some(Some("derivation" | "nixos-configuration")) => true,
                Some(Some(_) | None) => false,
                None => map.values().all(|v| !matches!(v, Value::Object(_))),
            },
            _ => false,
        }
    }

    pub async fn build_attributes(&self, attributes: &[String], dry_run: bool) -> Result<()> {
        if attributes.is_empty() {
            return Ok(());
        }

        let use_nom = nom_available().await;

        let bin = if use_nom { "nom" } else { "nix" };

        let mut args = vec!["build".to_string()];

        for attr in attributes {
            let build_attr = transform_attribute_for_build(attr);
            args.push(format!("{}#{}", self.flake_path, build_attr));
        }

        let mut command = Command::new(bin);
        command.args(&args);
        if dry_run {
            println!("dry-run: would execute:\n{:?}", command.as_std());
        } else {
            let mut child = command
                .stdin(Stdio::null())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()?;

            let status = child.wait().await?;

            if !status.success() {
                return Err(GarnixError::NixCommand(format!(
                    "{} build failed with exit code: {:?}",
                    bin,
                    status.code()
                )));
            }
        }

        Ok(())
    }
}

pub async fn nom_available() -> bool {
    Command::new("nom")
        .args(["--version"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false)
}

fn transform_attribute_for_build(attr: &str) -> String {
    if attr.starts_with("nixosConfigurations.") || attr.starts_with("darwinConfigurations.") {
        format!("{}.config.system.build.toplevel", attr)
    } else if attr.starts_with("homeConfigurations.") {
        format!("{}.activationPackage", attr)
    } else {
        attr.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_attribute_for_build() {
        assert_eq!(
            transform_attribute_for_build("nixosConfigurations.myhost"),
            "nixosConfigurations.myhost.config.system.build.toplevel"
        );

        assert_eq!(
            transform_attribute_for_build("darwinConfigurations.myhost"),
            "darwinConfigurations.myhost.config.system.build.toplevel"
        );

        assert_eq!(
            transform_attribute_for_build("homeConfigurations.myuser"),
            "homeConfigurations.myuser.activationPackage"
        );

        assert_eq!(
            transform_attribute_for_build("packages.x86_64-linux.hello"),
            "packages.x86_64-linux.hello"
        );

        assert_eq!(
            transform_attribute_for_build("checks.x86_64-linux.test"),
            "checks.x86_64-linux.test"
        );
    }
}
