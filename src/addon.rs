use std::{env, fs};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::{collections::HashMap, hash::Hash};

use futures::Future;
use reqwest::{Response};
use reqwest::header::{USER_AGENT, CONTENT_TYPE, HeaderMap, CONTENT_DISPOSITION};
use serde::{Deserialize, Serialize};
use serde_json::{Value};


// #[serde(skip_serializing)]

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AddonKind {
  GithubRelease,
  GithubRepo {branch: String},
  TukuiMain,
  TukuiAddon, 
  Gitlab,
  WowInt,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Addon {
  pub id: u32,
  pub project: String,
  pub name: Option<String>,
  pub version: Option<String>,
  pub dirs: Option<Vec<String>>,
  pub kind: AddonKind,
  pub download_url: Option<String>,
  pub filename: Option<String>,
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
      filename: None,
    }
  }

// cargo run -- -i https://github.com/Stanzilla/AdvancedInterfaceOptions https://github.com/Tercioo/Plater-Nameplates/tree/master https://gitlab.com/siebens/legacy/autoactioncam https://www.tukui.org/download.php?ui=tukui https://www.tukui.org/addons.php?id=209 https://www.wowinterface.com/downloads/info24608-HekiliPriorityHelper.html

// https://github.com/Stanzilla/AdvancedInterfaceOptions
// https://github.com/Tercioo/Plater-Nameplates/tree/master
// https://gitlab.com/siebens/legacy/autoactioncam
// https://www.tukui.org/download.php?ui=tukui
// https://www.tukui.org/addons.php?id=209
// https://www.wowinterface.com/downloads/info24608-HekiliPriorityHelper.html

  pub fn latest_url(&self) -> String {
    match &self.kind {
      AddonKind::GithubRelease => format!("https://api.github.com/repos/{}/releases/latest", self.project),
      AddonKind::GithubRepo{branch} => format!("https://api.github.com/repos/{}/commits/{}", self.project, branch),
      AddonKind::TukuiMain => format!("https://www.tukui.org/api.php?ui={}", self.project),
      AddonKind::TukuiAddon => String::from("https://www.tukui.org/api.php?addons"),
      AddonKind::Gitlab => format!("https://gitlab.com/api/v4/projects/{}/releases", self.project.replace("/", "%2F")),
      AddonKind::WowInt => format!("https://api.mmoui.com/v3/game/WOW/filedetails/{}.json", self.project),
    }
  }

  pub fn set_version(&mut self, json: &Value) {
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

  pub fn set_download_url(&mut self, json: &Value) {
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

  pub fn set_name(&mut self, json: &Value) {
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

  pub fn set_filename(&mut self, headers: &HeaderMap) {
    let filename = if headers.contains_key(CONTENT_DISPOSITION) {
      headers[CONTENT_DISPOSITION].to_str().unwrap()
        .split("=").into_iter().collect::<Vec<&str>>()
        .last().copied().unwrap().trim_matches(&['\'', '"'] as &[_])
    }
    else {
      self.download_url.as_ref().unwrap()
        .split("/").into_iter().collect::<Vec<&str>>()
        .last().copied().unwrap()
    };
    self.filename = Some(String::from(filename));
  }

  pub fn extract(&self) {
    let mut archive_path = env::temp_dir();
    archive_path.push(self.filename.as_ref().unwrap());
    
    let mut extract_path = env::temp_dir();
    extract_path.push(archive_path.file_stem().unwrap());

    let archive = File::open(archive_path).unwrap();
    zip_extract::extract(archive, &extract_path, false).unwrap();
    }
  
  pub fn install(&mut self) {
    let tmp = Path::new(self.filename.as_ref().unwrap());
    let mut extract_path = env::temp_dir();
    extract_path.push(tmp.file_stem().unwrap());

    let addon_dirs = move_addon_dirs(&extract_path);

  }  
  
  fn move_addon_dirs(extract_path: &PathBuf)  -> Vec<String> {
    let result: Vec<String> = Vec::new();

    // for file in fs::read_dir(&extract_path).unwrap() {
    //   let kind = file.unwrap().file_type().unwrap();
    //   if kind.is_file() {

    //   }

    fs::read_dir(extract_path)

    }
    // dig until we find a toc file
    // if this is the only directory at this level
      // if the name of dir is not the same as the toc file sans extension
        // add this dir and done
      // else
        // 
    // else
      // get all subdirs of the parent dir
        // 
    result
  }
}
