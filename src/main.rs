#[macro_use]
extern crate failure;

use std::collections::HashMap;

use cargo_metadata::{PackageId, Resolve};
use failure::Error;

#[derive(Default, Debug)]
struct Search {
    name_store: HashMap<PackageId, String>,
}

fn main() -> Result<(), Error> {
    let (target, other_flags) = {
        let mut args = std::env::args();
        let _ = args.next(); // we don't care about the name of the binary
        if args.next() != Some("why".to_string()) {
            usage();
            ::std::process::exit(1);
        }
        let target = match args.next() {
            Some(target) => {
                if target == "-h" || target == "--help" {
                    usage();
                    ::std::process::exit(1);
                }
                target
            }
            None => {
                usage();
                ::std::process::exit(1);
            }
        };
        let other_flags: Vec<String> = args.collect();
        (target, other_flags)
    };
    let mut cmd = cargo_metadata::MetadataCommand::new();
    cmd.other_options(other_flags);
    let metadata = cmd.exec()?;
    let resolve = match metadata.resolve {
        Some(x) => x,
        None => bail!("No dependency resolution found"),
    };

    let mut s = Search::default();
    for package in metadata.packages.iter() {
        s.name_store
            .insert(package.id.clone(), package.name.clone());
    }

    for root in metadata.workspace_members {
        s.search(vec![&root], &resolve, &target);
    }
    Ok(())
}

fn usage() {
    eprintln!(concat!(
        "cargo-why ",
        env!("CARGO_PKG_VERSION"),
        r#"

USAGE:
    cargo why <target crate> [other cargo flags (features, offline, etc)...]

FLAGS:
    -h, --help       Prints help information
"#
    ));
}

impl Search {
    fn search(&mut self, history: Vec<&PackageId>, resolve: &Resolve, target: &str) {
        let curr = match history.last() {
            Some(&x) => x,
            None => return,
        };
        if history[0..history.len() - 1].contains(&curr) {
            // avoid infinite recursion
            return;
        }
        let node = resolve.nodes.iter().find(|node| node.id == *curr);
        let node = match node {
            Some(x) => x,
            None => return,
        };
        for dep in &node.deps {
            if dep.name == target {
                for pkg in &history {
                    match self.name_store.get(pkg) {
                        Some(n) => {
                            print!("{} -> ", n);
                        }
                        None => {
                            // name lookup has failed fallback
                            // to full packageId.
                            print!("{} -> ", pkg);
                        }
                    }
                }
                println!("{}", target);
            } else {
                let mut history = history.clone();
                history.push(&dep.pkg);
                self.search(history, resolve, target);
            }
        }
    }
}
