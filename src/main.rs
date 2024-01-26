mod args;
mod include_line;
mod line_zero;
mod process;
mod system_paths;

use args::{Lang, Preprocessor, Protection};
use clap::Parser;
use process::process_lines;
use std::process::Command;

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
    #[arg(help = "path to c/c++ header file")]
    file: String,

    #[arg(
        default_value = Preprocessor::Cpp.as_str(),
        short = 'p',
        long = "preprocessor",
        value_enum
    )]
    preprocessor: Preprocessor,

    #[arg(default_value = Lang::Cpp.as_str(), short = 'x', long = "lang", value_enum)]
    lang: Lang,

    #[arg(
        long = "protect",
        default_value = "ifdef",
        help = "protect against multiple includes with `#ifdef` or `#pragma once`",
        value_enum
    )]
    protection: Protection,

    #[arg(help = r"additional parameters for the preprocessor")]
    cpp_opts: Vec<String>,
}

fn main() {
    let ops: Ops = Ops::parse();

    if let Err(e) = which::which(ops.preprocessor.as_str()) {
        panic!(
            "Failed to find preprocessor `{}`: {}",
            ops.preprocessor.as_str(),
            e
        );
    };

    let base_preprocessor_args = {
        let preprocessor_only = ops.preprocessor.required_args();

        let base_args = vec![
            "-x",
            ops.lang.as_str(),
            "-fdirectives-only", // prevent macro expansion
        ];

        preprocessor_only
            .into_iter()
            .chain(base_args.into_iter())
            .collect::<Vec<_>>()
    };

    let search_paths =
        system_paths::SearchPaths::new(ops.preprocessor.as_str(), &base_preprocessor_args);

    let extra_cpp_opts = ops.cpp_opts;

    let output = Command::new(ops.preprocessor.as_str())
        .args(&base_preprocessor_args)
        .arg(&ops.file)
        .args(extra_cpp_opts)
        .output()
        .expect("C preprocessor failed");

    if !output.status.success() {
        panic!(
            "C preprocessor exited with non-zero status code:\n{}",
            String::from_utf8(output.stderr)
                .expect("C preprocessor exited with non-zero and stderr isn't utf-8"),
        );
    }

    let txt = std::str::from_utf8(output.stdout.as_slice()).expect("cpp output isn't utf-8");
    let lines = txt.lines();

    ops.protection.protect(
        || {
            process_lines(lines, search_paths);
        },
        ops.file,
    );
}
