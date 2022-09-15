use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Source {
  Github, Tukui, Gitlab, WowInt
}

#[derive(Serialize, Deserialize, Debug)]
  pub struct UpdateData {


}

#[derive(Serialize, Deserialize, Debug)]
  pub struct Addon {
    id: Option<u32>,
    project: String,
    source: Source,
    #[serde(skip_serializing_if = "Option::is_none")]
    branch: Option<String>,
    name: Option<String>,
    version: Option<String>,
    dirs: Option<Vec<String>>,
    // #[serde(skip_serializing)]
    // latest_url: Option<String>,
    #[serde(skip_serializing)]
    download_url: Option<String>,
    #[serde(skip_serializing)]
    filename: Option<String>,
}

impl Addon {
  pub fn new(project: String, source: Source, branch: Option<String>) -> Self {
    Self { 
      project: project, 
      source: source,
      branch: branch,
      id: None,
      name: None,
      version: None,
      dirs: None,
      // latest_url: None,
      download_url: None,
      filename: None,
    }
  }

  pub fn get_latest(&mut self) {

  }

  pub fn letest_url(&self) -> String {
    match self.source {
      Source::Github => {
        match &self.branch {
          Some(b) => format!("https://api.github.com/repos/{}/commits/{}", self.project, b),
          None => format!("https://api.github.com/repos/{}/releases/latest", self.project)
        }
      },
      Source::Tukui => {
        if self.project == "tukui" || self.project == "elvui" {
          format!("https://www.tukui.org/api.php?ui={}", self.project)
        } else {
          String::from("https://www.tukui.org/api.php?addons")
        }
      },
      Source::Gitlab => {
        let url_encoded_project = self.project.replace("/", "%2F");
        format!("https://gitlab.com/api/v4/projects/{}/releases", url_encoded_project)
      }
      Source::WowInt => {
        format!("https://api.mmoui.com/v3/game/WOW/filedetails/{}.json", self.project)
      }
    }
  }
}