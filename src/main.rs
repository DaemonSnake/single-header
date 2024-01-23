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

    #[arg(default_value = Lang::Cpp.as_str(), short = 'x', long = "lang")]
    #[clap(value_enum)]
    lang: Lang,

    #[arg(
        long = "protect",
        default_value = "ifdef",
        help = "protect against multiple includes with `#ifdef` or `#pragma once`"
    )]
    #[clap(value_enum)]
    protection: Protection,

    #[arg(help = r"additional parameters for `cpp`")]
    cpp_opts: Option<Vec<String>>,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum Lang {
    C,
    #[clap(name = "c++")]
    Cpp,
}

impl Lang {
    fn as_str(&self) -> &'static str {
        match self {
            Lang::C => "c",
            Lang::Cpp => "c++",
        }
    }
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum Protection {
    Ifdef,
    Once,
}

fn pragme_once(action: impl FnOnce(), _filename: String) {
    println!("#pragma once");
    action();
}

fn ifndef_guard(action: impl FnOnce(), filename: String) {
    let invalid_macro_char = |c: char| !char::is_alphanumeric(c) && c != '_';

    let macro_name = filename.to_uppercase().replace(invalid_macro_char, "_");
    let macro_name = format!("{macro_name}_SINGLE_HEADER"); // prevent collisions with user-land include guards

    println!("#ifndef {macro_name}");
    println!("# define {macro_name}");
    action();
    println!("#endif // {macro_name}");
}

fn main() {
    let ops: Ops = Ops::parse();

    let extra_cpp_opts = ops.cpp_opts.unwrap_or_else(|| Vec::new());
    let default_cpp_opts = vec!["-fdirectives-only"];

    let output = Command::new("cpp")
        .arg("-x")
        .arg(ops.lang.as_str())
        .arg(&ops.file)
        .args(default_cpp_opts)
        .args(extra_cpp_opts)
        .output()
        .expect("C preprocessor failed");

    let txt = std::str::from_utf8(output.stdout.as_slice()).expect("cpp output isn't utf-8");
    let lines = txt.lines();

    let protection = match ops.protection {
        Protection::Ifdef => ifndef_guard,
        Protection::Once => pragme_once,
    };

    protection(
        || {
            process_lines(lines);
        },
        ops.file,
    );
}
