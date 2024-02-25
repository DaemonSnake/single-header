use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use radix_trie::{Trie, TrieCommon};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
struct CompileCommand {
    directory: String,
    command: Option<String>,
    arguments: Option<Vec<String>>,
    file: PathBuf,
    output: String,
}

fn supported_args_trie() -> Trie<String, ()> {
    let mut trie = Trie::new();
    let ops = vec!["-I", "-D", "-U", "-iquote", "-isystem", "-std="];
    for opt in ops {
        trie.insert(String::from(opt), ());
    }
    trie
}

lazy_static! {
    static ref PREPROCESSING_ARGS: Trie<String, ()> = supported_args_trie();
}

fn read_compile_commands(path: PathBuf, file: PathBuf) -> Result<Vec<String>> {
    let commands: Vec<CompileCommand> = serde_json::from_reader(File::open(path)?)?;

    let Some(command) = commands.into_iter().find(|cmd| file == cmd.file) else {
        return Ok(Vec::new());
    };

    let input_args = match command.arguments {
        Some(args) => args,
        None => {
            let cmd = command
                .command
                .ok_or(anyhow!("compile_commands.json: No command or arguments"))?;
            let args = shlex::split(&cmd)
                .ok_or(anyhow!("compile_commands.json: Failed to parse command"))?;
            args
        }
    };

    let mut saved_args = Vec::new();
    let mut include_next = false;

    for arg in input_args {
        if include_next {
            saved_args.push(arg);
            include_next = false;
            continue;
        }
        let Some(node) = PREPROCESSING_ARGS.get_ancestor(&arg) else {
            continue;
        };
        // if the arguments is as long as the key, then the value is the next argument
        if node.key().unwrap().len() == arg.len() {
            include_next = true;
        }
        saved_args.push(arg);
    }
    Ok(saved_args)
}

pub fn cmake_options(cmake_arg: Option<PathBuf>, file: &str) -> Result<Vec<String>> {
    let Some(cmake) = cmake_arg else {
        return Ok(Vec::new());
    };
    if !cmake.exists() {
        let err = anyhow!("cmake path does not exist: {}", cmake.display());
        return Err(err);
    }
    if !cmake.is_dir() {
        let err = anyhow!("cmake path doesn't point to a folder: {}", cmake.display());
        return Err(err);
    }

    let compile_commands = cmake.join("compile_commands.json");
    if !compile_commands.exists() {
        let err = anyhow!("compile_commands.json not found in {}", cmake.display());
        return Err(err);
    }

    let file = PathBuf::from(file).canonicalize()?; // absolute path to file
    read_compile_commands(compile_commands, file)
}
