use std::{collections::HashMap, hash::Hash};

use reqwest::{Response};
use reqwest::header::{USER_AGENT, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::{Value};
use core::future::{Future};

// #[serde(skip_serializing)]

#[derive(Serialize, Deserialize, Debug)]
pub enum AddonKind {
  GithubRelease,
  GithubRepo {branch: String},
  TukuiMain,
  TukuiAddon, 
  Gitlab,
  WowInt,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Addon {
  pub id: u32,
  pub project: String,
  pub name: Option<String>,
  pub version: Option<String>,
  pub dirs: Option<Vec<String>>,
  pub kind: AddonKind,
  #[serde(skip_serializing)]
  pub download_url: Option<String>,
}

const USER_AGENT_CHROME: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/105.0.0.0 Safari/537.36";

impl Addon {
  pub fn new(project: String, kind: AddonKind) -> Self {
    Self { 
      project: project, 
      id: 0,
      name: None,
      version: None,
      dirs: None,
      kind: kind,
      download_url: None,
    }
  }

// cargo run -- -i https://github.com/Stanzilla/AdvancedInterfaceOptions https://github.com/Tercioo/Plater-Nameplates/tree/master https://gitlab.com/siebens/legacy/autoactioncam https://www.tukui.org/download.php?ui=tukui https://www.tukui.org/addons.php?id=209 https://www.wowinterface.com/downloads/info24608-HekiliPriorityHelper.html

// https://github.com/Stanzilla/AdvancedInterfaceOptions
// https://github.com/Tercioo/Plater-Nameplates/tree/master
// https://gitlab.com/siebens/legacy/autoactioncam
// https://www.tukui.org/download.php?ui=tukui
// https://www.tukui.org/addons.php?id=209
// https://www.wowinterface.com/downloads/info24608-HekiliPriorityHelper.html


  #[tokio::main]
  pub async fn get_latest(&self) -> &dyn Future<Output = Result<Response, reqwest::Error>> {
    let url = self.latest_url();
    let client = reqwest::Client::new();
    
    &client
      .get(url)
      .header(CONTENT_TYPE, "application/json")
      .header(USER_AGENT, USER_AGENT_CHROME)
      .send()

    // self.set_version(&json);
    // self.set_download_url(&json);
    // self.set_name(&json);

    // println!("name: {}", self.name.as_deref().unwrap());
    // println!("version: {}", self.version.as_deref().unwrap());
    // println!("download_url: {}\n", self.download_url.as_deref().unwrap());
  }

  async fn get_updates(&mut self) {
    
  }

  fn latest_url(&self) -> String {
    match &self.kind {
      AddonKind::GithubRelease => format!("https://api.github.com/repos/{}/releases/latest", self.project),
      AddonKind::GithubRepo{branch} => format!("https://api.github.com/repos/{}/commits/{}", self.project, branch),
      AddonKind::TukuiMain => format!("https://www.tukui.org/api.php?ui={}", self.project),
      AddonKind::TukuiAddon => String::from("https://www.tukui.org/api.php?addons"),
      AddonKind::Gitlab => format!("https://gitlab.com/api/v4/projects/{}/releases", self.project.replace("/", "%2F")),
      AddonKind::WowInt => format!("https://api.mmoui.com/v3/game/WOW/filedetails/{}.json", self.project),
    }
  }

  fn set_version(&mut self, json: &Value) {
    self.version = match &self.kind {
      AddonKind::GithubRelease => {
        let v = json["tag_name"].as_str().unwrap();
        Some(String::from(if v != "" {v} else {json["name"].as_str().unwrap()}))
      },
      AddonKind::GithubRepo{..} => Some(String::from(json["sha"].as_str().unwrap())),
      AddonKind::TukuiMain => Some(String::from(json["version"].as_str().unwrap())),
      AddonKind::TukuiAddon => {
        let mut result: &str = "";
        for item in json.as_array().unwrap() {
          if item["id"].as_str().unwrap() == self.project {
            result = item["version"].as_str().unwrap();
          }
        }
        assert!(result != "");
        Some(String::from(result))
      },
      AddonKind::Gitlab => {
        let v = json[0]["tag_name"].as_str().unwrap();
        Some(String::from(if v != "" {v} else {json[0]["name"].as_str().unwrap()}))
      },
      AddonKind::WowInt => Some(String::from(json[0]["UIVersion"].as_str().unwrap())),
    }
  }

  fn set_download_url(&mut self, json: &Value) {
    self.download_url = Some(match &self.kind {
      AddonKind::GithubRelease => {
        let assets = json["assets"].as_array();
        match assets {
          Some(items) => {
            let mut result: &str = "";
            for item in items {
              if item["content_type"].as_str().unwrap() == "application/json" { continue }
              let url = item["browser_download_url"].as_str().unwrap();
              let lc = url.to_lowercase();
              if ["bcc", "tbc", "wotlk", "wrath", "classic"].iter().any(|&s| !lc.contains(s)) {
                result = url;
              }
            }
            assert!(result != "");
            String::from(result)
          },
          None => String::from(json["zipball_url"].as_str().unwrap())
        }
      },
      AddonKind::GithubRepo { branch } => format!("https://www.github.com/{}/archive/refs/heads/{}.zip", self.project, branch),
      AddonKind::TukuiMain => String::from(json["url"].as_str().unwrap()),
      AddonKind::TukuiAddon => {
        let mut result: &str = "";
        for item in json.as_array().unwrap() {
          if item["id"].as_str().unwrap() == self.project {
            result = item["url"].as_str().unwrap();
          }
        }
        assert!(result != "");
        String::from(result)
      },
      AddonKind::Gitlab => {
        let mut result: &str = "";
        let sources = json[0]["assets"]["sources"].as_array().unwrap();
        for s in sources {
          if s["format"].as_str().unwrap() == "zip" {
            result = s["url"].as_str().unwrap()
          }
        }
        String::from(result)
      },
      AddonKind::WowInt => String::from(json[0]["UIDownload"].as_str().unwrap()),
    });
  }

  fn set_name(&mut self, json: &Value) {
    self.name = Some(String::from(match &self.kind {
      AddonKind::GithubRelease | AddonKind::GithubRepo{..} | AddonKind::Gitlab => self.project.split('/').last().unwrap(),
      AddonKind::TukuiMain => json["name"].as_str().unwrap(),
      AddonKind::TukuiAddon => {
       let mut result: &str = "";
        for item in json.as_array().unwrap() {
          if item["id"].as_str().unwrap() == self.project {
            result = item["name"].as_str().unwrap();
          }
        }
        assert!(result != "");
        result
      },
      AddonKind::WowInt => json[0]["UIName"].as_str().unwrap(),
    }))
  }
}