use crate::include_line::{FlagStatus, IncludeDirective};

pub struct LineZeroState {
    in_line_zero: bool,
}

pub struct Skip(pub bool);

impl LineZeroState {
    pub fn new() -> Self {
        LineZeroState {
            in_line_zero: false,
        }
    }

    pub fn ignore_line(&self) -> bool {
        return self.in_line_zero;
    }

    // handle builtin `0` lines produced by the processor
    pub fn feed(&mut self, include_info: &IncludeDirective) -> Skip {
        if include_info.linenum != 0 && !self.in_line_zero {
            return Skip(false);
        }

        // only know it's the last builtin line 0 when `# 0 "..." 2` is found
        // any other line starting with `# 0 "..."` indicates we are still in a built-in section
        // and should continue ignoring all lines

        if !self.in_line_zero {
            self.in_line_zero = true;
        } else if matches!(include_info.state.status, FlagStatus::Close) {
            self.in_line_zero = false;
        }
        return Skip(true);
    }
}
