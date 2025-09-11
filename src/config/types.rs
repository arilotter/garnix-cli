use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct GarnixConfig {
    #[serde(default)]
    pub builds: BuildsConfig,

    #[serde(default, rename = "incrementalizeBuilds")]
    pub incrementalize_builds: IncrementalizeBuilds,

    #[serde(default)]
    pub servers: Vec<ServerConfig>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum BuildsConfig {
    Single(BuildEntry),
    Multiple(Vec<BuildEntry>),
}

#[derive(Debug, Clone, Deserialize)]
pub struct BuildEntry {
    #[serde(default = "default_includes")]
    pub include: Vec<String>,

    #[serde(default)]
    pub exclude: Vec<String>,

    pub branch: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum IncrementalizeBuilds {
    Boolean(bool),
    ExcludesBranches { exclude_branches: Vec<String> },
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub configuration: String,
    pub deployment: DeploymentConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum DeploymentConfig {
    #[serde(rename = "on-pull-request")]
    OnPullRequest,

    #[serde(rename = "on-branch")]
    OnBranch { branch: String },
}

impl Default for BuildsConfig {
    fn default() -> Self {
        BuildsConfig::Single(BuildEntry::default())
    }
}

impl Default for BuildEntry {
    fn default() -> Self {
        BuildEntry {
            include: default_includes(),
            exclude: Vec::new(),
            branch: None,
        }
    }
}

impl Default for IncrementalizeBuilds {
    fn default() -> Self {
        IncrementalizeBuilds::Boolean(false)
    }
}

fn default_includes() -> Vec<String> {
    vec![
        "*.x86_64-linux.*".to_string(),
        "defaultPackage.x86_64-linux".to_string(),
        "devShell.x86_64-linux".to_string(),
        "homeConfigurations.*".to_string(),
        "darwinConfigurations.*".to_string(),
        "nixosConfigurations.*".to_string(),
    ]
}

impl BuildsConfig {
    pub fn entries(&self) -> Vec<&BuildEntry> {
        match self {
            BuildsConfig::Single(entry) => vec![entry],
            BuildsConfig::Multiple(entries) => entries.iter().collect(),
        }
    }
}
