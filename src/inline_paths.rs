use radix_trie::Trie;
use std::path::{Path, PathBuf};

pub struct InlinePaths {
    paths: Trie<PathBuf, ()>,
}

impl InlinePaths {
    pub fn new(paths: Vec<String>) -> Self {
        let mut trie = Trie::new();
        for path in paths {
            match Path::new(&path).canonicalize() {
                Ok(p) => trie.insert(p, ()),
                Err(e) => panic!(
                    "invalid inline path: {path}, failed to retrieve absolute path with err: {e}"
                ),
            };
        }
        InlinePaths { paths: trie }
    }

    pub fn should_inline(&self, path: &PathBuf) -> bool {
        self.paths.get_ancestor(path).is_some()
    }
}
