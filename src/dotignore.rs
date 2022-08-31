use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::{Path, PathBuf};

use log;
use regex::RegexSet;

pub struct DotIgnoreConfig {
    filename: PathBuf,
    parser: DotIgnoreConfigParser,
    ignore_globs: Vec<String>,
}

struct DotIgnoreConfigParser {
    comment_delimeter: char,
}

impl DotIgnoreConfig {
    pub fn new(filename: &Path) -> DotIgnoreConfig {
        DotIgnoreConfig {
            filename: PathBuf::from(filename),
            parser: DotIgnoreConfigParser::defaults(),
            ignore_globs: Vec::new(),
        }
    }

    pub fn collect(&mut self) {
        self.ignore_globs = self.parser.load(&self.filename).unwrap();
        log::debug!("Collected dotignore globs: {:?}", self.ignore_globs);
    }

    pub fn match_glob(&self, path: &Path) -> bool {
        let re_set = RegexSet::new(&self.ignore_globs).unwrap();
        log::debug!(
            "Attempting to match `{}` to ignored globs: {:?}",
            path.display(),
            re_set.patterns()
        );

        !re_set.matches(path.to_str().unwrap()).matched_any()
    }
}

impl DotIgnoreConfigParser {
    pub fn new(comment_delimeter: char) -> Self {
        Self { comment_delimeter }
    }

    pub fn defaults() -> Self {
        Self {
            comment_delimeter: '#',
        }
    }

    fn load(&mut self, path: &Path) -> Result<Vec<String>, String> {
        log::debug!("Loading dotignore file `{}`", path.display());

        let pathname = path.to_str().unwrap();
        let filename = path.file_name().unwrap().to_str().unwrap();

        if !path.exists() {
            log::debug!(
                "File `{}` .dotignore not found, loading no ignored globs",
                path.display()
            );
            return Ok(Vec::new());
        }

        let file = match File::open(&path) {
            Err(why) => return Err(format!("Couldn't open {}: {}", pathname, why)),
            Ok(file) => file,
        };

        let buf_reader = BufReader::new(file);

        let mut ignore_globs = match self.read(buf_reader) {
            Err(why) => return Err(format!("Couldn't read file {}: {}", pathname, why)),
            Ok(ignored_glob) => ignored_glob,
        };

        // Add .dotignore file to `ignored_globs`
        ignore_globs.push(String::from(filename));

        Ok(ignore_globs)
    }

    fn partition(s: String, delimeter: char) -> (String, String) {
        match s.split_once(delimeter) {
            Some(tup) => (tup.0.to_string(), tup.1.to_string()),
            None => (s, "".to_string()),
        }
    }

    fn read(&self, reader: BufReader<File>) -> Result<Vec<String>, String> {
        let lines = reader.lines();
        let mut globs = Vec::new();

        for line in lines {
            let line = match line {
                Ok(line) => line,
                Err(why) => why.to_string(),
            };

            // Partion .0 -> parseable, .1 -> comment
            let glob = Self::partition(line, self.comment_delimeter)
                .0
                .trim()
                .to_string();

            // Ignore newlines
            if !glob.is_empty() {
                globs.push(glob);
            }
        }
        Ok(globs)
    }
}
