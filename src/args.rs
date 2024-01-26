#[derive(clap::ValueEnum, Clone, Debug)]
pub enum Preprocessor {
    Cpp,
    Gcc,
    Clang,
}

impl Preprocessor {
    pub fn as_str(&self) -> &'static str {
        match self {
            Preprocessor::Cpp => "cpp",
            Preprocessor::Gcc => "gcc",
            Preprocessor::Clang => "clang",
        }
    }

    pub fn required_args(&self) -> Vec<&'static str> {
        match self {
            Preprocessor::Cpp => vec![],
            Preprocessor::Gcc | Preprocessor::Clang => vec!["-E"],
        }
    }
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum Lang {
    C,
    #[clap(name = "c++")]
    Cpp,
}

impl Lang {
    pub fn as_str(&self) -> &'static str {
        match self {
            Lang::C => "c",
            Lang::Cpp => "c++",
        }
    }
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum Protection {
    Ifndef,
    Once,
}

impl Protection {
    pub fn protect(&self, action: impl FnOnce(), filename: String) {
        match self {
            Protection::Ifndef => ifndef_guard(action, filename),
            Protection::Once => pragme_once(action),
        }
    }
}

fn pragme_once(action: impl FnOnce()) {
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
