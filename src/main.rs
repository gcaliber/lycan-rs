// TODO:
// parse command line arguments ::clap
// install
// remove
// update
// beautify output

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use futures::Future;
use reqwest::Response;
use serde_json::{Result, Value};

use futures::{stream, StreamExt};
use reqwest::Client;
use tokio;

use regex::Regex;
use std::fs::{File, OpenOptions};

use std::io::Write;
use std::path::{Path, PathBuf};
use std::fs;

use clap::Parser;

use crate::addon::{Addon, AddonKind};
mod addon;

use crate::config::{Config};
mod config;

use std::env;

use reqwest::header::{USER_AGENT, CONTENT_TYPE, CONTENT_DISPOSITION};
const CHROME_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/105.0.0.0 Safari/537.36";

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
	/// List of addons to install.
	#[clap(short, long, value_parser, multiple_values = true, group = "action")]
	install: Option<Vec<String>>,

	/// List of addon IDs to remove.
	#[clap(short, long, value_parser, multiple_values = true, group = "action")]
	remove: Option<Vec<u32>>
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let config = Config::new(r"/home/mike/projects/addons/");
	let cli = Cli::parse();

	let mut addons: Vec<Addon> = Vec::new();
	for url in cli.install.unwrap() {
		if let Some(a) = addon_from_url(&url) {
			addons.push(a);
		}
	}

	const PARALLEL_REQUESTS: usize = 10;

	let client = Client::new();
	let updates = stream::iter(addons)
		.map(|addon| {
			let url = addon.latest_url();
			let client = client.clone();
			tokio::spawn(async move {
				let json = client.get(url)
				.header(CONTENT_TYPE, "application/json")
				.header(USER_AGENT, CHROME_USER_AGENT)
				.send()
				.await.unwrap()
				.json::<Value>()
				.await;
				(addon, json)
			})
		})
		.buffer_unordered(PARALLEL_REQUESTS);

	let acc: Vec<Addon> = Vec::new();
	let addons_updated = updates
		.fold(acc, |mut acc, u| async {
			match u {
				Ok((mut addon, json)) => {
					match json {
						Ok(json) => {
							addon.set_version(&json);
							addon.set_download_url(&json);
							addon.set_name(&json);
							acc.push(addon);
						},
						Err(e) => eprintln!("Got a reqwest::Error {}", e),		
					}
				},
				Err(e) => eprintln!("Got a tokio::JoinError: {}", e),
			}
			acc
		})
		.await;

	let downloads = stream::iter(addons_updated)
		.map(|mut addon| {
			let client = client.clone();
			tokio::spawn(async move {
				let download_url = addon.download_url.as_ref().unwrap();
				let resp = client.get(download_url)
				.header(USER_AGENT, CHROME_USER_AGENT)
				.send()
				.await.unwrap();

				addon.set_filename(resp.headers());
				let bytes = resp.bytes().await;
				(addon, bytes)
			})
		})
		.buffer_unordered(PARALLEL_REQUESTS);

	let acc: Vec<Addon> = Vec::new();
	let addons_downloaded = downloads
		.fold(acc, |mut acc, u| async {
			match u {
				Ok((addon, bytes)) => {
					match bytes {
						Ok(b) => {
							let mut path = env::temp_dir();
							path.push(addon.filename.as_ref().unwrap());
							path.set_extension("zip");
							let mut f = File::create(&path).expect(format!("Error opening file: {:?}", &path).as_str());
							f.write_all(&b).expect(format!("Error writing file: {:?}", &path).as_str());
							acc.push(addon)
						},
						Err(e) => eprintln!("Got an error while downloading: {}", e),
					}
				},
				Err(e) => eprintln!("Got a tokio::JoinError: {}", e),
			}
			acc
		})
		.await;

	let addons_installed: Vec<Addon> = addons_downloaded.into_iter()
		.flat_map(|mut addon| {
			addon.install(&config)?;
			anyhow::Ok(addon)
		}).collect();








	// let addons_json = PathBuf::from(r"/home/mike/projects/lycan/test/addons.json");
	// let installed_addons = read_addons(&addons_json).unwrap(); 	

	// let addons_finalized = addons_downloaded.into_iter()
	// 	.map(|mut addon| {
	// 		addon.set_id(&installed_addons)
	// 	});


	Ok(())
}

fn addon_from_url(url: &String) -> Option<Addon> {
	// implement lazy_static on the regex's here for speed
	let re = Regex::new(r#"^(?:https?://)?(?:www\.)?(?P<domain>.*)\.(?:com|org)/(?P<rest>.*)"#).unwrap();
	let caps = re.captures(url)?;
	let domain = caps.name("domain")?.as_str();
	let rest = caps.name("rest")?.as_str();
	match domain {
		"github" => {
			let re = Regex::new(r#"^(?P<project>.+?/.+?)(?:/|$)(?:tree/)?(?P<branch>.+)?"#).unwrap();
			let caps = re.captures(rest)?;
			let project = caps.name("project")?.as_str();
			Some(match caps.name("branch") {
				Some(m) => Addon::new(String::from(project), AddonKind::GithubRepo{branch: String::from(m.as_str())}),
				None => Addon::new(String::from(project), AddonKind::GithubRelease),
			})
		},
		"gitlab" => Some(Addon::new(String::from(rest), AddonKind::Gitlab)),
		"tukui" => {
			let re = Regex::new(r#"^(?P<kind>download|addons)\.php\?(?:ui|id)=(?P<project>.*)"#).unwrap();
			let caps = re.captures(rest)?;
			let project = caps.name("project")?.as_str();
			let kind = caps.name("kind")?.as_str();
			match kind {
				"download" => Some(Addon::new(String::from(project), AddonKind::TukuiMain)),
				"addons" => Some(Addon::new(String::from(project), AddonKind::TukuiAddon)),
				&_ => None,
			}
		},
		"wowinterface" => {
			let re = Regex::new(r#"^downloads/info(?P<project>\d*)-"#).unwrap();
			let caps = re.captures(rest)?;
			let project = caps.name("project")?.as_str();
			Some(Addon::new(String::from(project), AddonKind::WowInt))
		},
		&_ => None
	}
}


// rewrite as a method for addon?
fn write_addons(addons: &Vec<Addon>, path: &PathBuf) -> anyhow::Result<()> {
	let s = serde_json::to_string_pretty(addons)?;
	fs::write(path, s)?;
	Ok(())
}

fn read_addons(path: &PathBuf) -> anyhow::Result<Vec<Addon>> {
	let s = fs::read_to_string(path)?;
	let a: Vec<Addon> = serde_json::from_str(&s)?;
	Ok(a)
}

fn test() -> Result<()> {
	// let addons_json = PathBuf::from(r"/home/mike/projects/lycan/test/addons.json");
	// let addons = read_addons(&addons_json).unwrap();
	let test_json = PathBuf::from(r"/home/mike/projects/lycan/test/test.json");


	let mut a = Addon::new(String::from("test/addon"), AddonKind::GithubRepo { branch: String::from("master") });
	a.id = 1;
	a.name = Some(String::from("addon"));
	a.version = Some(String::from("v1.0"));
	a.dirs = Some(vec![String::from("dir1"), String::from("dir2")]);

	let mut b = Addon::new(String::from("test/addon222"), AddonKind::Gitlab);
	b.id = 1;
	b.name = Some(String::from("addon222"));
	b.version = Some(String::from("v1.0222"));
	b.dirs = Some(vec![String::from("dir1"), String::from("dir2")]);

	let aa = vec![a, b];

	// let addons = read_addons(&addons_json).unwrap();

	write_addons(&aa, &test_json).unwrap();

	Ok(())
}
