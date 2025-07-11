
use eyre::ensure;
use proc_macro::TokenStream;
use std::{ops::Not, process::Command, str::FromStr};

fn get_git_commit() -> eyre::Result<String> {
    let output = Command::new("git")
        .args(["log", "-1", "--pretty=format:%h,%ad", "--date=format:%Y-%m-%d"])
        .output()?;
    let output_str = std::str::from_utf8(&output.stdout)?.trim().to_string();
    let pairs: Vec<&str> = output_str.split(",").collect();
    ensure!(pairs.len() == 2, "Unexpected output format");

    // get dirty status
    let dirty_output = Command::new("git")
        .args(["status", "-s"])
        .output()?;
    let dirty = std::str::from_utf8(&dirty_output.stdout)?
        .is_empty()
        .not()
        .then(|| "-dirty".to_string())
        .unwrap_or_default();

    let branch_output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()?;
    let branch = std::str::from_utf8(&branch_output.stdout)?.trim().to_string();

    Ok(format!(
        "\"{branch}-{commit}{dirty}@{date}\"",
        commit = pairs[0],
        date = pairs[1]
    ))
}

#[proc_macro]
pub fn build_version(_item: TokenStream) -> TokenStream {
    let version = get_git_commit().unwrap();
    TokenStream::from_str(version.as_str()).unwrap()
}
