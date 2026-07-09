use std::collections::VecDeque;
use std::path::{Path, PathBuf};

/// 1:1 translation of `com.fumbbl.ffb.util.FileIterator`.
///
/// Breadth-first filesystem iterator; returns files (and optionally directories)
/// under a root directory that match a predicate.
pub struct FileIterator {
    /// Java: fStartDirectory
    start_directory: PathBuf,
    /// Java: fIncludeDirectories
    include_directories: bool,
    /// Java: file filter predicate
    file_filter: Box<dyn Fn(&Path) -> bool + Send + Sync>,
    /// BFS queue of pending directories to descend into
    directory_queue: VecDeque<PathBuf>,
    /// Files ready to return from the current level
    file_queue: VecDeque<PathBuf>,
    /// Java: fKnownSize
    known_size: usize,
}

impl FileIterator {
    /// Convenience: accept all, include directories.
    pub fn new(start: impl AsRef<Path>) -> Self {
        Self::with_options(start, true, |_| true)
    }

    pub fn with_include_dirs(start: impl AsRef<Path>, include_directories: bool) -> Self {
        Self::with_options(start, include_directories, |_| true)
    }

    pub fn with_options(
        start: impl AsRef<Path>,
        include_directories: bool,
        filter: impl Fn(&Path) -> bool + Send + Sync + 'static,
    ) -> Self {
        let start_directory = start.as_ref().to_path_buf();
        let mut s = Self {
            start_directory: start_directory.clone(),
            include_directories,
            file_filter: Box::new(filter),
            directory_queue: VecDeque::from(vec![start_directory]),
            file_queue: VecDeque::new(),
            known_size: 0,
        };
        s.descend();
        s
    }

    /// Java: `knownSize()`.
    pub fn known_size(&self) -> usize { self.known_size }

    /// Java: `hasNext()`.
    pub fn has_next(&mut self) -> bool {
        while self.file_queue.is_empty() && !self.directory_queue.is_empty() {
            self.descend();
        }
        !self.file_queue.is_empty()
    }

    /// Java: `next()`.
    pub fn next(&mut self) -> Option<PathBuf> {
        if self.has_next() { self.file_queue.pop_front() } else { None }
    }

    /// Java: `reset()`.
    pub fn reset(&mut self) {
        self.directory_queue.clear();
        self.directory_queue.push_back(self.start_directory.clone());
        self.file_queue.clear();
        self.known_size = 0;
        self.descend();
    }

    /// Java: `descend()`.
    fn descend(&mut self) {
        let mut next_dirs = VecDeque::new();
        while let Some(dir) = self.directory_queue.pop_front() {
            if self.include_directories && dir != self.start_directory {
                self.file_queue.push_back(dir.clone());
                self.known_size += 1;
            }
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() && (self.file_filter)(&path) {
                        self.file_queue.push_back(path);
                        self.known_size += 1;
                    } else if path.is_dir() {
                        next_dirs.push_back(path);
                    }
                }
            }
        }
        self.directory_queue = next_dirs;
    }
}

impl Iterator for FileIterator {
    type Item = PathBuf;
    fn next(&mut self) -> Option<Self::Item> { FileIterator::next(self) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn iterates_files_in_temp_dir() {
        let dir = std::env::temp_dir().join("ffb_file_iter_test");
        let _ = fs::create_dir_all(&dir);
        let file_path = dir.join("test.txt");
        let mut f = fs::File::create(&file_path).unwrap();
        let _ = f.write_all(b"test");
        drop(f);

        let mut iter = FileIterator::new(&dir);
        let found: Vec<PathBuf> = iter.by_ref().collect();
        assert!(found.iter().any(|p| p.file_name().map(|n| n == "test.txt").unwrap_or(false)));

        let _ = fs::remove_file(&file_path);
        let _ = fs::remove_dir(&dir);
    }

    #[test]
    fn known_size_counts_files() {
        let dir = std::env::temp_dir().join("ffb_file_iter_size");
        let _ = fs::create_dir_all(&dir);
        let f1 = dir.join("a.txt");
        let f2 = dir.join("b.txt");
        let _ = fs::write(&f1, b"a");
        let _ = fs::write(&f2, b"b");

        let mut iter = FileIterator::new(&dir);
        while iter.has_next() { iter.next(); }
        assert!(iter.known_size() >= 2);

        let _ = fs::remove_file(&f1);
        let _ = fs::remove_file(&f2);
        let _ = fs::remove_dir(&dir);
    }
}
