use anyhow::{Context, Result};
use std::{
    io::BufRead,
    process::{Command, Output},
};

// Anyhow tools
macro_rules! lazy_context {
    ($expr:expr, $format:tt, $($arg:tt)*) => { Context::with_context($expr, || format!($format, $($arg)*)) }
}

// convert IntoIter of std:::result::Result into anyhow::Result<Vec>
pub trait TryResFold {
    type Item;
    fn try_res_fold(self) -> anyhow::Result<Vec<Self::Item>>;
}

impl<Item, Err, I> TryResFold for I
where
    I: IntoIterator<Item = std::result::Result<Item, Err>>,
    Err: std::error::Error + std::marker::Sync + std::marker::Send + 'static,
{
    type Item = Item;

    fn try_res_fold(self) -> anyhow::Result<Vec<Self::Item>> {
        let try_fold = |mut vec: Vec<Item>, line| -> anyhow::Result<Vec<Item>> {
            vec.push(line?);
            anyhow::Result::Ok(vec)
        };
        self.into_iter().try_fold(Vec::new(), try_fold)
    }
}

// Command tools
fn run_command(description: &'static str, mut command: Command) -> Result<Output> {
    let output = lazy_context!(command.output(), "Failed to run {}", description)?;

    if !output.status.success() {
        panic!(
            "C preprocessor exited with non-zero status code:\n{}",
            String::from_utf8(output.stderr)
                .expect("C preprocessor exited with non-zero and stderr isn't utf-8"),
        );
    }
    Ok(output)
}

pub fn stdout_command(description: &'static str, command: Command) -> Result<Vec<String>> {
    let output = run_command(description, command)?;
    output.stdout.lines().try_res_fold()
}

pub fn stderr_command(description: &'static str, command: Command) -> Result<Vec<String>> {
    let output = run_command(description, command)?;
    output.stderr.lines().try_res_fold()
}

pub trait LazyForEach {
    type Item;
    fn lazy_for_each<Action>(self, action: Action) -> impl FnOnce()
    where
        Action: FnMut(Self::Item);
}

impl<Item, I> LazyForEach for I
where
    I: IntoIterator<Item = Item>,
{
    type Item = Item;

    fn lazy_for_each<Action>(self, action: Action) -> impl FnOnce()
    where
        Action: FnMut(Self::Item),
    {
        move || self.into_iter().for_each(action)
    }
}
