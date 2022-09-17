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

use crate::addon::{Addon, AddonKind};
mod addon;
// https://github.com/Tercioo/Plater-Nameplates


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

	test().unwrap()

	// let addons_json = PathBuf::from(r"/home/mike/projects/lycan/test/addons.json");
	// let installed_addons = match read_addons(&addons_json) {
	// 	Ok(a) => a,
	// 	Err(e) => Vec::new(),
	// };

	


	// if let Some(urls) = cli.install.as_deref() {
	// 	let mut addons: Vec<Option<Addon>> = Vec::new();
	// 	for url in urls {
	// 		addons.push(addon_from_url(url));
	// 	}

	// 	for addon in addons.iter().flatten() {
	// 		match addon.get_latest() {
	// 			Ok(_) => continue,
	// 			Err(e) => println!("{}", e)
	// 		}
	// 		// get latest json
	// 		// add update info from latest json
	// 		// download file
	// 		// unpack file
	// 		// parse addon top level directories
	// 		// move addon dirs from temp to destination
	// 		// write installed addons to file
	// 	}

	// }
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
