use anyhow::Result;
use std::collections::VecDeque;
use std::path::PathBuf;

use crate::include_line::{self, IncludeDirective};
use crate::inline_paths::InlinePaths;
use crate::line_zero::{LineZeroState, Skip};
use crate::system_paths::SearchPaths;

use include_line::FlagStatus;

pub fn process_lines<I: IntoIterator<Item = String>>(
    lines: I,
    search_paths: SearchPaths,
    inline_paths: InlinePaths,
) -> Vec<String> {
    let mut output = vec![];
    let mut p = Processor::new(search_paths, inline_paths);
    for line in lines {
        if let Some(output_line) = p.process_line(line.as_str()) {
            output.push(output_line);
        }
    }
    output
}

struct ShowContent(bool);

struct Processor {
    search_paths: SearchPaths,
    inline_paths: InlinePaths,
    include_queue: VecDeque<ShowContent>,
    line_zero: LineZeroState,
}

impl Processor {
    fn new(search_paths: SearchPaths, inline_paths: InlinePaths) -> Self {
        Processor {
            search_paths,
            inline_paths,
            include_queue: VecDeque::new(),
            line_zero: LineZeroState::new(),
        }
    }

    fn process_line<'a>(&mut self, line: &'a str) -> Option<String> {
        match include_line::try_parse(line) {
            None => {
                // ignore builtin defines and includes
                if self.line_zero.ignore_line() {
                    return None;
                }
                if matches!(self.include_queue.back(), Some(ShowContent(false))) {
                    return None;
                }
                Some(String::from(line))
            }
            Some(include_info) => {
                if let Skip(true) = self.line_zero.feed(&include_info) {
                    return None;
                }
                if include_info.state.ignorable() {
                    return None;
                }
                self.try_undo_system_include(include_info)
            }
        }
    }

    fn try_undo_system_include(&mut self, include_info: IncludeDirective) -> Option<String> {
        let state = include_info.state;

        let Some(path) = include_info.absolute_path else {
            panic!(
                "include file {} in cpp output doesn't exists",
                include_info.filename
            );
        };
        let system_header = state.system_header && !self.inline_paths.should_inline(&path);
        match state.status {
            FlagStatus::Open => {
                // replace content of system header with its include directive
                // don't hide local headers

                let is_hidding_included_lines =
                    matches!(self.include_queue.back(), Some(ShowContent(false)));

                let ret = if system_header && !is_hidding_included_lines {
                    let include = self
                        .system_include_string(&path)
                        .expect("Failed to create system include string from absolute path");
                    Some(include)
                } else {
                    None
                };

                let include_state = ShowContent(!system_header);
                self.include_queue.push_back(include_state);

                return ret;
            }
            FlagStatus::Close => {
                if !self.include_queue.is_empty() {
                    self.include_queue.pop_back();
                }
            }
            _ => {}
        };
        None
    }

    fn system_include_string(&self, filename: &PathBuf) -> Result<String> {
        let include_name = self.search_paths.cleanup_path(filename);

        match include_name {
            Err(err) => {
                let ctx = format!("Failed to cleanup path: {}", filename.display());
                Err(err.context(ctx))
            }
            Ok(include_name) => {
                let include = format!("#include <{include_name}>");
                Ok(include)
            }
        }
    }
}
