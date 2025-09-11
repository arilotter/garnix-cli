use crate::config::{BuildEntry, BuildsConfig, GarnixConfig};
use crate::error::{GarnixError, Result};
use std::collections::HashSet;

pub struct AttributeMatcher {
    current_branch: String,
}

impl AttributeMatcher {
    pub fn new(current_branch: String) -> Self {
        Self { current_branch }
    }

    pub fn get_matching_attributes(
        &self,
        config: &GarnixConfig,
        available_attrs: &[String],
    ) -> Result<Vec<String>> {
        let config = match config {
            GarnixConfig::Null => return Ok(Vec::new()),
            GarnixConfig::Config(config) => config,
        };

        let mut matched_attrs = HashSet::new();

        let applicable_entries = self.get_applicable_build_entries(&config.builds);

        for entry in applicable_entries {
            self.apply_include_patterns(&entry.include, available_attrs, &mut matched_attrs)?;
            self.apply_exclude_patterns(&entry.exclude, available_attrs, &mut matched_attrs)?;
        }

        let mut result: Vec<String> = matched_attrs.into_iter().collect();
        result.sort();
        Ok(result)
    }

    fn get_applicable_build_entries<'a>(&self, builds: &'a BuildsConfig) -> Vec<&'a BuildEntry> {
        builds
            .entries()
            .into_iter()
            .filter(|entry| {
                match &entry.branch {
                    None => true, // No branch filter means applies to all branches
                    Some(branch) => branch == &self.current_branch,
                }
            })
            .collect()
    }

    fn matches_pattern(&self, pattern: &str, attr: &str) -> Result<bool> {
        let pattern_parts: Vec<&str> = pattern.split('.').collect();
        let attr_parts: Vec<&str> = attr.split('.').collect();

        if pattern_parts.len() != attr_parts.len() {
            return Ok(false);
        }

        if !matches!(pattern_parts.len(), 2 | 3) {
            return Err(GarnixError::PatternMatch(format!(
                "Invalid pattern format '{}', must be 'x.y' or 'x.y.z'",
                pattern
            )));
        }

        for (pattern_part, attr_part) in pattern_parts.iter().zip(attr_parts.iter()) {
            if *pattern_part != "*" && *pattern_part != *attr_part {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn apply_include_patterns(
        &self,
        patterns: &[String],
        available_attrs: &[String],
        matched_attrs: &mut HashSet<String>,
    ) -> Result<()> {
        for pattern in patterns {
            for attr in available_attrs {
                if self.matches_pattern(pattern, attr)? {
                    matched_attrs.insert(attr.clone());
                }
            }
        }
        Ok(())
    }

    fn apply_exclude_patterns(
        &self,
        patterns: &[String],
        available_attrs: &[String],
        matched_attrs: &mut HashSet<String>,
    ) -> Result<()> {
        for pattern in patterns {
            for attr in available_attrs {
                if self.matches_pattern(pattern, attr)? {
                    matched_attrs.remove(attr);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{BuildEntry, BuildsConfig};

    #[test]
    fn test_pattern_matching() {
        let matcher = AttributeMatcher::new("main".to_string());

        // Test exact matches
        assert!(
            matcher
                .matches_pattern("packages.x86_64-linux.hello", "packages.x86_64-linux.hello")
                .unwrap()
        );

        // Test wildcard matches
        assert!(
            matcher
                .matches_pattern("packages.*.*", "packages.x86_64-linux.hello")
                .unwrap()
        );
        assert!(
            matcher
                .matches_pattern("*.x86_64-linux.*", "packages.x86_64-linux.hello")
                .unwrap()
        );
        assert!(
            matcher
                .matches_pattern("packages.x86_64-linux.*", "packages.x86_64-linux.hello")
                .unwrap()
        );

        // Test non-matches
        assert!(
            !matcher
                .matches_pattern("packages.aarch64-linux.*", "packages.x86_64-linux.hello")
                .unwrap()
        );
        assert!(
            !matcher
                .matches_pattern("checks.*.*", "packages.x86_64-linux.hello")
                .unwrap()
        );

        // Test different number of parts
        assert!(
            !matcher
                .matches_pattern("packages.*", "packages.x86_64-linux.hello")
                .unwrap()
        );
        assert!(
            !matcher
                .matches_pattern("packages.*.*.*", "packages.x86_64-linux.hello")
                .unwrap()
        );

        // Test 2-part patterns
        assert!(
            matcher
                .matches_pattern("devShell.*", "devShell.x86_64-linux")
                .unwrap()
        );
        assert!(
            matcher
                .matches_pattern("defaultPackage.x86_64-linux", "defaultPackage.x86_64-linux")
                .unwrap()
        );
    }

    #[test]
    fn test_branch_filtering() {
        let matcher = AttributeMatcher::new("main".to_string());

        let main_entry = BuildEntry {
            include: vec!["packages.*.*".to_string()],
            exclude: vec![],
            branch: Some("main".to_string()),
        };

        let dev_entry = BuildEntry {
            include: vec!["checks.*.*".to_string()],
            exclude: vec![],
            branch: Some("dev".to_string()),
        };

        let no_branch_entry = BuildEntry {
            include: vec!["devShells.*".to_string()],
            exclude: vec![],
            branch: None,
        };

        let builds = BuildsConfig::Multiple(vec![main_entry, dev_entry, no_branch_entry]);
        let applicable = matcher.get_applicable_build_entries(&builds);

        assert_eq!(applicable.len(), 2); // main entry + no_branch entry
        assert!(
            applicable
                .iter()
                .any(|e| e.include.contains(&"packages.*.*".to_string()))
        );
        assert!(
            applicable
                .iter()
                .any(|e| e.include.contains(&"devShells.*".to_string()))
        );
        assert!(
            !applicable
                .iter()
                .any(|e| e.include.contains(&"checks.*.*".to_string()))
        );
    }
}
