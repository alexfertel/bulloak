use std::{
    env,
    path::PathBuf,
    process::{Command, Output},
};

pub(crate) fn get_binary_path() -> PathBuf {
    let root = env::current_exe()
        .unwrap()
        .parent()
        .expect("should be in the executable's directory")
        .to_path_buf();
    root.join("../bulloak")
}

/// Runs a command with the specified args.
#[allow(dead_code)] // Not used in all test crates.
pub(crate) fn cmd(
    binary_path: &PathBuf,
    command: &str,
    tree_path: &PathBuf,
    args: &[&str],
) -> Output {
    Command::new(binary_path)
        .arg(command)
        .arg(tree_path)
        .args(args)
        .output()
        .expect("should execute the command")
}
