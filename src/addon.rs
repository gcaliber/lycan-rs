use std::{collections::HashMap, hash::Hash};

use reqwest::header::{USER_AGENT, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Source {
  Github, Tukui, Gitlab, WowInt
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
    #[serde(skip_serializing)]
    download_url: Option<String>,
    #[serde(skip_serializing)]
    filename: Option<String>,
}

const USER_AGENT_CHROME: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/105.0.0.0 Safari/537.36";

trait GetLatest {
  fn get_latest(&self) -> anyhow::Result<()>;
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
      download_url: None,
      filename: None,
    }
  }

  #[tokio::main]
  pub async fn get_latest(&self) -> anyhow::Result<()> {
    let url = self.latest_url();
    let client = reqwest::Client::new();
    // let resp = reqwest::get(url)
    //     .await?
    //     .json::<HashMap<String, String>>()
    //     .await?;
    let json = client
      .get(url)
      .header(CONTENT_TYPE, "application/json")
      .header(USER_AGENT, USER_AGENT_CHROME)
      .send()
      .await?.json::<serde_json::Value>().await;

    // println!("version: {}");
    // println!("download_url: {}");

    // let resp = reqwest::get(url).await?;
    // let text = resp.text().await?;
    // println!("{}", text);
    // let json = serde_json::from_str(&text)?;
    // println!("{:#?}", json);
    println!("{:#?}", json);
    Ok(())
  }

  fn latest_url(&self) -> String {
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

  fn set_version(&mut self, json: serde_json::Value) {
    match self.source {
      Source::Github => {
        if self.branch == None {
          let v = json["tag_name"].as_str().unwrap();
          self.version = Some(String::from(if v != "" {v} else {json["name"].as_str().unwrap()}))
        } else {
          self.version = Some(String::from(json["sha"].as_str().unwrap()));
        }
      },
      Source::Tukui => {
        self.version = Some(String::from(json["version"].as_str().unwrap()));
      },
      Source::Gitlab => {
        let v = json[0]["tag_name"].as_str().unwrap();
        self.version = Some(String::from(if v != "" {v} else {json[0]["name"].as_str().unwrap()}))
      }
      Source::WowInt => {
        self.version = Some(String::from(json[0]["UIversion"].as_str().unwrap()));
      }
    }
  }

  fn set_download_url(&mut self, json: serde_json::Value) {
    self.download_url = Some(match self.source {
      Source::Github => {
        if self.branch == None {
          let assets = json["assets"].as_array();
          match assets {
            Some(items) => {
              let mut result: &str;
              for item in items {
                let url = item["browser_download_url"].as_str().unwrap();
                let lc = url.to_lowercase();
                if ["bcc", "tbc", "wotlk", "wrath", "classic"].iter().any(|&s| lc.contains(s)) {
                  result = url;
                } else {
                  result = "";
                }
              }
              String::from(result)
            },
            None => String::from(json["zipball_url"].as_str().unwrap())
          }
        } else {
          format!("https://www.github.com/{}/archive/refs/heads/{}.zip", self.project, self.branch.unwrap())
        }
      },
      
      Source::Tukui => String::from(json["url"].as_str().unwrap()),
      Source::Gitlab => {
        let mut result: &str = "";
        for s in json[0]["assets"]["sources"].as_object() {
          if s["format"].as_str().unwrap() == "zip" {
            result = s["url"].as_str().unwrap()
          }
        }
        String::from(result)
      }
      Source::WowInt => { "".to_string()

      }
    });
  }


}