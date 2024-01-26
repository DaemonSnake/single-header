use std::path::{Path, PathBuf};

use lazy_static::lazy_static;
use regex::Regex;

// Documentation: https://gcc.gnu.org/onlinedocs/cpp/Preprocessor-Output.html

pub enum FlagStatus {
    NotSet,
    Open = 1,
    Close = 2,
}

enum FlagOpt {
    SystemHeader = 3,
    ExternC = 4,
}

pub struct IncludeState {
    pub status: FlagStatus,
    pub system_header: bool,
    pub extern_c: bool,
}

impl IncludeState {
    fn new() -> Self {
        IncludeState {
            status: FlagStatus::NotSet,
            system_header: false,
            extern_c: false,
        }
    }

    pub fn ignorable(&self) -> bool {
        match self.status {
            // include output directive with no flag values can be ignored
            //`# linenum "..."`
            FlagStatus::NotSet => true,
            _ => false,
        }
    }
}

pub struct IncludeDirective {
    pub linenum: u32,
    pub filename: String,
    pub absolute_path: Option<PathBuf>,
    pub state: IncludeState,
}

fn make_include_output_regex() -> Regex {
    let hash_start = r"^#";
    let sep = r"\s*";
    let line_regex = r"(\d+)";
    let filename_regex = r#""([^"]*)""#;

    let re = format!(r"{hash_start}{sep}{line_regex}{sep}{filename_regex}");
    return Regex::new(re.as_str()).unwrap();
}

fn make_flags_regex() -> Regex {
    let flag_regex = r"\s*(\d+)";
    return Regex::new(flag_regex).unwrap();
}

lazy_static! {
    static ref INCLUDE_OUTPUT_REGEX: Regex = make_include_output_regex();
    static ref FLAGS_REGEX: Regex = make_flags_regex();
}

pub fn try_parse(line: &str) -> Option<IncludeDirective> {
    let Some(captures) = INCLUDE_OUTPUT_REGEX.captures(line) else {
        return None;
    };
    let (full, [linenum, filename]) = captures.extract();
    let linenum = linenum.parse::<u32>().unwrap();
    let end = full.len();
    let flags_substr = &line[end..];

    let mut state = IncludeState::new();

    FLAGS_REGEX.captures_iter(flags_substr).for_each(|c| {
        let (_, [flag_str]) = c.extract();
        let flag_number = flag_str.parse::<u32>().unwrap();

        match flag_number {
            flag if flag == FlagStatus::Open as u32 => {
                state.status = FlagStatus::Open;
            }
            flag if flag == FlagStatus::Close as u32 => {
                state.status = FlagStatus::Close;
            }
            flag if flag == FlagOpt::SystemHeader as u32 => {
                state.system_header = true;
            }
            flag if flag == FlagOpt::ExternC as u32 => {
                state.extern_c = true;
            }
            _ => {}
        }
    });
    let absolute_path = Path::new(filename).canonicalize().ok();
    let filename = String::from(filename);

    return Some(IncludeDirective {
        linenum,
        filename,
        absolute_path,
        state,
    });
}
