use std::collections::{HashSet, VecDeque};
use std::fs::DirEntry;
use std::io::Error;
use std::path::{Path, PathBuf};
use std::vec::Vec;

pub trait Crawler {
    fn crawl(&mut self) -> Result<Vec<PathBuf>, String>;
}
pub struct FileCrawler {
    cwd: PathBuf,
}

pub struct DirectoryCrawler {
    cwd: PathBuf,
}

impl FileCrawler {
    pub fn new(cwd: &Path) -> Self {
        FileCrawler {
            cwd: PathBuf::from(cwd),
        }
    }
}

impl DirectoryCrawler {
    pub fn new(cwd: &Path) -> Self {
        DirectoryCrawler {
            cwd: PathBuf::from(cwd),
        }
    }
}

impl Crawler for FileCrawler {
    fn crawl(&mut self) -> Result<Vec<PathBuf>, String> {
        let mut files = Vec::new();
        log::debug!("Crawling directory `{}` for files", self.cwd.display());

        let dir_entries = match self.cwd.read_dir() {
            Err(why) => return Err(format!("Couldn't read directory: {}", why)),
            Ok(entries) => entries,
        };

        for entry in dir_entries {
            // let pathname = entry.expect("Error").file_name().to_str().unwrap();
            let path = entry.unwrap().path();
            log::debug!("Found directory entry `{}`", path.display());
            if path.is_file() {
                log::debug!("Adding `{}` to crawled files", path.display());
                files.push(path);
            }
        }
        Ok(files)
    }
}

impl Crawler for DirectoryCrawler {
    fn crawl(&mut self) -> Result<Vec<PathBuf>, String> {
        let mut folders = Vec::new();
        log::debug!(
            "Crawling top-level directory `{}` for directories",
            self.cwd.display()
        );

        // Perform DFS for directories
        let mut dir_stack = VecDeque::new();
        let mut discovered = HashSet::new();

        let dir_entries = match self.cwd.read_dir() {
            Err(why) => return Err(format!("Couldn't read directory: {}", why)),
            Ok(entries) => entries,
        };

        dir_entries
            .filter(|e| e.as_ref().unwrap().path().is_dir())
            .for_each(|e| dir_stack.push_back(e.unwrap().path()));

        while let Some(path) = dir_stack.pop_back() {
            log::debug!(
                "Crawling directory `{}` for directories",
                path.to_str().unwrap()
            );

            if !discovered.contains(&path) {
                discovered.insert(path.clone());

                let subdir_entries: Vec<Result<DirEntry, Error>> = match path.read_dir() {
                    Err(why) => return Err(format!("Couldn't read directory: {}", why)),
                    Ok(entries) => entries.collect(),
                };

                subdir_entries
                    .iter()
                    .filter(|e| e.as_ref().unwrap().path().is_dir())
                    .for_each(|e| dir_stack.push_back(e.as_ref().unwrap().path()));

                for e in subdir_entries {
                    if e.unwrap().file_name().into_string().unwrap() == String::from(".dotfileln") {
                        log::debug!("Adding `{}` to crawled files", path.clone().display());
                        folders.push(path.clone());
                    }
                }
                // TODO Determine why this does not work
                /*
                if subdir_entries.any(|e| {
                    e.unwrap().file_name().into_string().unwrap() == String::from(".dotfileln")
                }) {
                    log::debug!("Adding `{}` to crawled files", path.display());
                    folders.push(path);
                }*/
            }
        }
        Ok(folders)
    }
}
