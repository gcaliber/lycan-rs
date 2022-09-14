// TODO:
// parse command line arguments ::clap
// install
// remove
// update
// beautify output

use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};

use std::path::{Path, PathBuf};
use std::fs;

#[derive(Serialize, Deserialize, Debug)]
enum Source {
    Github, Tukui, Gitlab, GithubRepo, WowInt
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateData {
    latest_url: String,
    download_url: String,
    version: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Addon {
    id: u16,
    project: String,
    source: Source,
    #[serde(skip_serializing_if = "Option::is_none")]
    branch: Option<String>,
    name: String,
    version: String,
    dirs: Vec<String>,
    #[serde(skip_serializing)]
    update: Option<UpdateData>
}
// https://github.com/Tercioo/Plater-Nameplates

fn main() {
    test().unwrap()
}

fn write_json(text: &Vec<Addon>, path: &PathBuf) -> anyhow::Result<()> {
    let s = serde_json::to_string_pretty(text)?;
    fs::write(path, s)?;
    Ok(())
}

fn read_addons(path: &PathBuf) -> anyhow::Result<Vec<Addon>> {
    let s = fs::read_to_string(path)?;
    let a: Vec<Addon> = serde_json::from_str(&s)?;
    Ok(a)
}

fn test() -> Result<()> {
    let addons_json = PathBuf::from(r"/home/mike/projects/lycan/test/addons.json");
    let test_json = PathBuf::from(r"/home/mike/projects/lycan/test/test.json");

    let addons = read_addons(&addons_json).unwrap();

    write_json(&addons, &test_json).unwrap();

    Ok(())
}
