mod include_line;
mod line_zero;
mod process;

use clap::Parser;
use process::process_lines;
use std::process::Command;

#[derive(Parser, Debug)]
#[command(
    author = "DaemonSnake",
    about = "convert C/C++ file into portable single-header file",
    long_about = r"
Convert C/C++ file into portable single-header file

Runs C/C++ (cpp) preprocessor on <FILE>.
It then undoes the `#include` expension of all the system headers.
It replaces them with an `#include` directive as close as possible to the original one.
As it only has access to the fully-qualified path at that point, it replaces it with the base filename.
Might not work for all cases.
"
)]
struct Ops {
    #[arg(help = "path to c/c++ header file")]
    file: String,

    #[arg(default_value = "c++", short = 'x', long = "lang")]
    lang: String,

    #[arg(
        long = "protect",
        default_value = "ifdef",
        help = "protect against multiple includes with `#ifdef` or `#pragma once`"
    )]
    protection: String,

    #[arg(help = r"additional parameters for `cpp`")]
    cpp_opts: Option<Vec<String>>,
}

fn main() {
    let ops: Ops = Ops::parse();

    let extra_cpp_opts = ops.cpp_opts.unwrap_or_else(|| Vec::new());
    let default_cpp_opts = vec!["-fdirectives-only"];

    let output = Command::new("cpp")
        .arg("-x")
        .arg(ops.lang)
        .arg(&ops.file)
        .args(default_cpp_opts)
        .args(extra_cpp_opts)
        .output()
        .expect("C preprocessor failed");

    let txt = std::str::from_utf8(output.stdout.as_slice()).expect("cpp output isn't utf-8");
    let lines = txt.lines();

    let invalid_macro_char = |c: char| !char::is_alphanumeric(c) && c != '_';

    let def_name = ops.file.to_uppercase().replace(invalid_macro_char, "_");
    let def_name = format!("{def_name}_SINGLE_HEADER"); // prevent collisions with user-land include guards
    let is_pragma = matches!(ops.protection.as_str(), "once");

    {
        if is_pragma {
            println!("#pragma once");
        } else {
            println!("#ifndef {def_name}");
            println!("# define {def_name}");
        }
    }
    process_lines(lines);
    {
        if !is_pragma {
            println!("#endif // {def_name}")
        }
    }
}
