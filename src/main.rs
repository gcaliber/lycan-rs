// TODO:
// parse command line arguments ::clap
// install
// remove
// update
// beautify output

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use serde_json::{Result, Value};

use regex::Regex;

use std::path::{Path, PathBuf};
use std::fs;

use clap::Parser;

use crate::addon::Addon;
use crate::addon::Source;
mod addon;
// https://github.com/Tercioo/Plater-Nameplates


// mutually exclusive options
// #[clap(group = "input")]
// input_file: Option<String>,

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
fn main() {
	let cli = Cli::parse();

	let addons_json = PathBuf::from(r"/home/mike/projects/lycan/test/addons.json");
	let installed_addons = match read_addons(&addons_json) {
		Ok(a) => a,
		Err(e) => Vec::new(),
	};


	if let Some(urls) = cli.install.as_deref() {
		let mut addons: Vec<Option<Addon>> = Vec::new();
		for url in urls {
			addons.push(addon_from_url(url));
		}

		for addon in addons.iter().flatten() {
			// create update with latest json url
			// get latest json
			// add update info from latest json
			// download file
			// unpack file
			// parse addon top level directories
			// move addon dirs from temp to destination
			// write installed addons to file
		}

	}
}


fn addon_from_url(url: &String) -> Option<Addon> {
	// implement lazy_static on the regex's here for speed
	let re = Regex::new(r"^(?:https?:\/\/)?(?:www\.)?(?P<domain>.*)\.(?:com|org)\/(?P<rest>.*[^\/\n])").unwrap();
	let caps = re.captures(url)?;
	let domain = caps.name("domain")?.as_str();
	let rest = caps.name("rest")?.as_str();

	match domain {
		"github" => { 
			let re = Regex::new(r"^(?P<project>.+\/.+)\/tree\/(?P<branch>.+)").unwrap();
			let caps = re.captures(rest)?;
			let project = caps.name("project")?.as_str();
			let branch = match caps.name("branch") {
				Some(m) => Some(String::from(m.as_str())),
				None => None
			};
			Some(Addon::new(String::from(project), Source::Github, branch))
		},
		"gitlab" => Some(Addon::new(String::from(rest), Source::Gitlab, None)),
		"tukui" => {
			let re = Regex::new(r"^(?:download|addons)\.php\?(?:ui|id)=(?P<project>.*)").unwrap();
			let caps = re.captures(rest)?;
			let project = caps.name("project")?.as_str();
			Some(Addon::new(String::from(project), Source::Tukui, None))
		},
		"wowinterface" => {
			let re = Regex::new(r"^downloads\/info(?P<project>\d*)-").unwrap();
			let caps = re.captures(rest)?;
			let project = caps.name("project")?.as_str();
			Some(Addon::new(String::from(project), Source::WowInt, None))
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
	let addons_json = PathBuf::from(r"/home/mike/projects/lycan/test/addons.json");
	let test_json = PathBuf::from(r"/home/mike/projects/lycan/test/test.json");

	let addons = read_addons(&addons_json).unwrap();

	write_addons(&addons, &test_json).unwrap();

	Ok(())
}
