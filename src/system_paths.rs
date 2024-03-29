use crate::utils::stderr_command;
use anyhow::{anyhow, Result};
use radix_trie::{Trie, TrieCommon};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

pub struct SearchPaths {
    search_paths: Trie<PathBuf, ()>,
}

impl SearchPaths {
    pub fn new<'a, 'b>(
        program: &'a str,
        base_args: &Vec<&'b str>,
        extra_args: &Vec<String>,
    ) -> Result<SearchPaths> {
        let mut command = Command::new(program);

        command
            .args(base_args)
            .arg("/dev/null") // dummy file to make preprocessor immediately exit
            .arg("-v")
            .args(extra_args);

        let stderr_lines = stderr_command("C preprocessor", command)?;
        let mut parsing_search_list: bool = false;

        let mut search_paths = Trie::new();

        for line in stderr_lines {
            if line.starts_with("#include <...> search starts here:") {
                parsing_search_list = true;
                continue;
            }
            if line.starts_with("End of search list.") {
                break;
            }

            if parsing_search_list {
                let line = line.trim(); // remove indentation
                let path = Path::new(line)
                    .canonicalize() // convert to absolute path
                    .expect("preprocessor search path doesn't exists");
                search_paths.insert(path, ());
            }
        }

        Ok(SearchPaths { search_paths })
    }

    pub fn cleanup_path(&self, absolute_path: &PathBuf) -> Result<String> {
        let Some(search_path_trie) = self.search_paths.get_ancestor(absolute_path) else {
            return Err(anyhow!(
                "Path {} is not in search paths",
                absolute_path.display()
            ));
        };
        let prefix = search_path_trie.key().unwrap(); // unwrap is safe because we know the path is in the trie
        let stripped_path = absolute_path
            .strip_prefix(prefix)
            .expect("search path prefix was found but couldn't be stripped")
            .display()
            .to_string();

        Ok(stripped_path)
    }
}
