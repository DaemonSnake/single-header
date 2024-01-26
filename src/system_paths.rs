use radix_trie::{Trie, TrieCommon};
use std::{
    io::BufRead,
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
    ) -> SearchPaths {
        let output = Command::new(program)
            .args(base_args)
            .arg("/dev/null") // dummy file to make preprocessor immediately exit
            .arg("-v")
            .args(extra_args)
            .output()
            .expect(
                "C preprocessor failed to be called with -v, required to retrieve system paths",
            );

        if !output.status.success() {
            panic!(
                "C preprocessor exited with non-zero status code:\n{}",
                String::from_utf8(output.stderr)
                    .expect("C preprocessor exited with non-zero and stderr isn't utf-8"),
            );
        }

        let mut parsing_search_list = false;
        let debug_lines = output.stderr.lines().map(|l| l.unwrap());

        let mut search_paths: Trie<_, _> = Trie::new();

        for line in debug_lines {
            if line.starts_with("#include <...> search starts here:") {
                parsing_search_list = true;
                continue;
            } else if line.starts_with("End of search list.") {
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

        SearchPaths { search_paths }
    }

    pub fn cleanup_path(&self, path: &str) -> String {
        let absolute_path = Path::new(path)
            .canonicalize()
            .expect("failed to convert header to absulute path");

        let Some(search_path_trie) = self.search_paths.get_ancestor(&absolute_path) else {
            panic!("Path {} is not in search paths", path);
        };
        absolute_path
            .strip_prefix(search_path_trie.key().unwrap())
            .expect("search path prefix was found but couldn't be stripped")
            .to_str()
            .expect("result path isn't valid unicode")
            .to_string()
    }
}
