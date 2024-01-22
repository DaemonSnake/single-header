use std::collections::VecDeque;

use crate::include_line::{self, IncludeDirective};
use crate::line_zero::{LineZeroState, Skip};

use include_line::FlagStatus;

pub fn process_lines<'a, I: Iterator<Item = &'a str>>(lines: I) {
    let mut p = Processor::new();
    for line in lines {
        p.process_line(line)
    }
}

struct ShowContent(bool);

struct Processor {
    include_queue: VecDeque<ShowContent>,
    line_zero: LineZeroState,
}

impl Processor {
    fn new() -> Self {
        Processor {
            include_queue: VecDeque::new(),
            line_zero: LineZeroState::new(),
        }
    }

    fn process_line<'a>(&mut self, line: &'a str) {
        match include_line::try_parse(line) {
            None => {
                // ignore builtin defines and includes
                if self.line_zero.ignore_line() {
                    return;
                }
                if matches!(self.include_queue.back(), Some(ShowContent(false))) {
                    return;
                }
                println!("{line}");
            }
            Some(include_info) => {
                if let Skip(true) = self.line_zero.feed(&include_info) {
                    return;
                }
                if include_info.state.ignorable() {
                    return;
                }
                self.on_include_info(include_info)
            }
        }
    }

    fn on_include_info(&mut self, include_info: IncludeDirective) {
        let state = include_info.state;
        let filename = include_info.filename;

        match state.status {
            FlagStatus::Open => {
                // replace content of system header with its include directive
                // don't hide local headers

                let is_hidding_included_lines =
                    matches!(self.include_queue.back(), Some(ShowContent(false)));

                if state.system_header && !state.extern_c && !is_hidding_included_lines {
                    print_include(filename);
                }

                let include_state = ShowContent(!state.system_header);
                self.include_queue.push_back(include_state);
            }
            FlagStatus::Close => {
                if !self.include_queue.is_empty() {
                    self.include_queue.pop_back();
                }
            }
            _ => {}
        }
    }
}

fn print_include<S: AsRef<str>>(filename: S) {
    let path = std::path::Path::new(filename.as_ref());

    if !path.is_file() {
        panic!("header path is not a file {}", filename.as_ref())
    }

    let filename = path.file_name().unwrap().to_str().unwrap();
    println!("#include<{filename}>");
}
