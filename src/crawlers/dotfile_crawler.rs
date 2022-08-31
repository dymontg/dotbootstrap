use crate::crawlers::crawler::{Crawler, DirectoryCrawler, FileCrawler};

use std::path::{Path, PathBuf};

pub struct DotCrawler {
    dotfiles_crawler: FileCrawler,
    dotfolders_crawler: DirectoryCrawler,
}

impl DotCrawler {
    pub fn new(dot_dir: &Path) -> DotCrawler {
        DotCrawler {
            dotfiles_crawler: FileCrawler::new(dot_dir),
            dotfolders_crawler: DirectoryCrawler::new(dot_dir),
        }
    }

    pub fn crawl(&mut self) -> Result<Vec<PathBuf>, String> {
        log::info!("Crawling dotfiles directory");
        let mut crawled = self.dotfiles_crawler.crawl()?;
        // Symlinked dotfolders have a .dotfileln file in their root dir.
        crawled.extend(self.dotfolders_crawler.crawl()?);

        Ok(crawled)
    }
}
