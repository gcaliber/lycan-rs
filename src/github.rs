use crate::addon::{Source, GetLatest};
mod addon;

#[derive(Serialize, Deserialize, Debug)]
  pub struct GithubAddon {
    id: u32,
    project: String,
    source: Source,
    #[serde(skip_serializing_if = "Option::is_none")]
    branch: Option<String>,
    name: Option<String>,
    version: Option<String>,
    dirs: Option<Vec<String>>,
    #[serde(skip_serializing)]
    download_url: Option<String>,
    #[serde(skip_serializing)]
    filename: Option<String>,
}

const USER_AGENT_CHROME: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/105.0.0.0 Safari/537.36";

impl GithubAddon {
  pub fn new(project: String, source: Source, branch: Option<String>) -> Self {
    Self { 
      project: project,
      source: source,
      branch: branch,
      id: 0,
      name: None,
      version: None,
      dirs: None,
      download_url: None,
      filename: None,
    }
  }
}

impl GetLatest for GithubAddon {

}