mod args;
mod cmake;
mod include_line;
mod inline_paths;
mod line_zero;
mod process;
mod system_paths;
mod utils;

use anyhow::Result;
use args::{Lang, Preprocessor, Protection};
use clap::{ArgAction, Parser};
use process::process_lines;
use std::process::Command;
use utils::LazyForEach;

#[derive(Parser, Debug)]
#[command(
    author = "DaemonSnake",
    about = "convert C/C++ file into portable single-header file",
    long_about = r#"
Convert C/C++ file into portable single-header file

Runs C/C++ <preprocessor> on <FILE>
Preventing builtin macros and macro expansion (but #if/#ifdef will be executed).
It then undoes the `#include` expension of all the system headers,
replacing them with an `#include <...>` directive that will be portable.
"#
)]
struct Ops {
    #[arg(
        default_value = Preprocessor::Cpp.as_str(),
        short = 'p',
        long = "preprocessor",
        value_enum
    )]
    preprocessor: Preprocessor,

    #[arg(
        long = "cmake",
        help = "path to build folder to find the compile_commands.json file"
    )]
    cmake: Option<std::path::PathBuf>,

    #[arg(
        short='i',
        long="inline",
        name="INLINE_PATH",
        action = ArgAction::Append,
        help="path / file that must allways be `#include` expanded (can provided multiple times)"
    )]
    inline_paths: Vec<String>,

    #[arg(default_value = Lang::Cpp.as_str(), short = 'x', long = "lang", value_enum)]
    lang: Lang,

    #[arg(
        long = "protect",
        default_value = "ifndef",
        help = "protect against multiple includes with `#ifndef` or `#pragma once`",
        value_enum
    )]
    protection: Protection,

    #[arg(help = "path to c/c++ header file")]
    file: String,

    #[arg(
        help = r"additional parameters for the preprocessor",
        last = true,
        action = ArgAction::Append,
    )]
    cpp_opts: Vec<String>,
}

fn base_args(required: Vec<&'static str>, lang: Lang) -> Vec<&'static str> {
    let base_args = vec![
        "-x",
        lang.as_str(),
        "-fdirectives-only", // prevent macro expansion
    ];

    utils::merge(required, base_args)
}

fn main() -> Result<()> {
    let ops = Ops::try_parse()?;

    if let Err(e) = which::which(ops.preprocessor.as_str()) {
        panic!(
            "Failed to find preprocessor `{}`: {}",
            ops.preprocessor.as_str(),
            e
        );
    };

    let base_preprocessor_args = base_args(ops.preprocessor.required_args(), ops.lang);

    let cmake_opts = cmake::cmake_options(ops.cmake, &ops.file)?;
    let extra_cpp_opts = utils::merge(cmake_opts, ops.cpp_opts);

    let search_paths = system_paths::SearchPaths::new(
        ops.preprocessor.as_str(),
        &base_preprocessor_args,
        &extra_cpp_opts,
    )?;

    let inline_paths = inline_paths::InlinePaths::new(ops.inline_paths);

    let mut command = Command::new(ops.preprocessor.as_str());

    command
        .args(&base_preprocessor_args)
        .arg(&ops.file)
        .args(extra_cpp_opts);

    let lines = utils::stdout_command("C preprocessor", command)?;
    let output = process_lines(lines, search_paths, inline_paths);

    let lazy_print_lines = output.lazy_for_each(|line| println!("{line}"));
    ops.protection.protect(lazy_print_lines, ops.file);

    Ok(())
}
