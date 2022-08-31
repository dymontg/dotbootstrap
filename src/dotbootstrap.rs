use crate::crawlers::dotfile_crawler::DotCrawler;

use crate::dotignore::DotIgnoreConfig;

use std::path::{Path, PathBuf};

use std::collections::HashMap;
use std::env;
use std::io::stdin;
use std::os::unix;

use colored::*;
use log;

pub struct DotBootstrap {
    dotignore: DotIgnoreConfig,
    crawler: DotCrawler,
    src_dir: PathBuf,
    dest_dir: PathBuf,
}

impl DotBootstrap {
    pub fn new(dotignore_fn: &Path, src_dir: &Path, dest_dir: &Path) -> DotBootstrap {
        DotBootstrap {
            dotignore: DotIgnoreConfig::new(dotignore_fn),
            crawler: DotCrawler::new(src_dir),
            src_dir: src_dir.to_path_buf(),
            dest_dir: dest_dir.to_path_buf(),
        }
    }

    fn get_exec_name() -> Result<String, String> {
        match env::current_exe() {
            Ok(exec) => match exec.file_name() {
                Some(filename) => Ok(filename.to_str().unwrap().to_string()),
                None => Err(String::from("No filename")),
            },
            Err(why) => Err(why.to_string()),
        }
    }

    fn collect_map(&mut self) -> HashMap<PathBuf, PathBuf> {
        log::info!("Collecting map of linkable files");
        let mut map = HashMap::new();
        // Read .dotignore
        self.dotignore.collect();
        let linkable = self
            .crawler
            .crawl()
            .unwrap()
            .into_iter()
            .filter(|pathbuf| self.dotignore.match_glob(pathbuf));

        let exec_name = Self::get_exec_name().unwrap();
        for from in linkable {
            // Don't include executable in collection
            if !from
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .ends_with(&exec_name)
            {
                log::debug!("Found linkable file {}", from.display());
                let to_path = self
                    .dest_dir
                    .clone()
                    .join(from.strip_prefix(&self.src_dir).unwrap());
                // TODO is .clone() the right thing to do here?
                map.insert(from.clone(), to_path);
            }
        }

        map
    }

    pub fn dryrun(&mut self) {
        log::info!("Running a dryrun of a dotfile bootstrap");
        let map = self.collect_map();
        for (from, to) in map.iter() {
            if to.exists() {
                log::debug!("source file {} already exists. Warning user.", to.display());
                println!(
                    "{}",
                    format!(
                        "{} -> {} (Destination exists!)",
                        from.display(),
                        to.display()
                    )
                    .bold()
                    .red()
                );
            } else {
                println!(
                    "{}",
                    format!("{} -> {}", from.display(), to.display()).green()
                );
            }
        }
    }

    pub fn bootstrap(&mut self) {
        log::info!("Performing a dotfile bootstrap");
        let map = self.collect_map();
        for (from, to) in map.iter() {
            if to.exists() {
                log::debug!("source file {} already exists. Warning user.", to.display());
                let symlinked = match ask_yn(format!(
                        "Attempting to symlink {} -> {}, but the desitation already exists. Overwite? [y/N]",
                        from.display(),
                        to.display()
                    )
                    .bold()
                    .yellow().to_string()
                ) {
                    Err(_) => {
                        println!(
                            "{}",
                            format!("Not linking {} -> {}", from.display(), to.display()).red()
                        );
                        Ok(())
                    },
                    Ok(_) => Self::symlink(&from, &to)
                    }
                ;
                symlinked.unwrap();
            } else {
                Self::symlink(from, to).unwrap();
            }
        }
    }

    // Greedy symlink function
    fn symlink(target: &Path, linkname: &Path) -> Result<(), String> {
        if linkname.exists() {
            if linkname.is_file() {
                // TODO what should be returned when the match leg type doesnt matter?
                match std::fs::remove_file(linkname) {
                    Err(why) => return Err(why.to_string()),
                    Ok(_) => "",
                };
            } else {
                match std::fs::remove_dir_all(linkname) {
                    Err(why) => return Err(why.to_string()),
                    Ok(_) => "",
                };
            }
        }

        match unix::fs::symlink(target, linkname) {
            Err(why) => Err(why.to_string()),
            Ok(_) => {
                println!(
                    "{}",
                    format!("Symlinked {} -> {}", target.display(), linkname.display()).green()
                );
                Ok::<(), String>(())
            }
        }
    }
}

fn ask_yn(question: String) -> Result<(), ()> {
    let mut line_buf = String::new();
    let mut answered = None;

    println!("{}", question);
    while answered.is_none() {
        line_buf.clear();
        stdin().read_line(&mut line_buf).unwrap();
        line_buf = line_buf.trim().to_ascii_lowercase();
        log::debug!("User answered: `{}`", line_buf);

        answered = match line_buf.as_str() {
            "y" | "yes" => Some(Ok(())),
            "n" | "no" | _ => Some(Err(())),
        };
    }
    return answered.unwrap();
}
