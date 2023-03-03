//! ```
//! NAME
//!         stale-autolabel
//!
//! SYNOPSIS
//!         stale-autolabel [FILE]
//!
//! DESCRIPTION
//!         Detect stale paths in autolabel definitions in triagebot.toml.
//!         Probably autofix them in the future.
//!
//!         It is assumed to be executed under the same directory where
//!         triagebot.toml resides.
//! ```

use std::path::Path;
use toml_edit::Document;

fn main() {
    let path = std::env::args_os()
        .skip(1)
        .next()
        .unwrap_or("triagebot.toml".into());

    let mut failed = 0;
    let mut passed = 0;

    let toml = std::fs::read_to_string(path).expect("read from file");
    let doc = toml.parse::<Document>().expect("a toml");
    let autolabel = doc["autolabel"].as_table().expect("a toml table");

    for (label, value) in autolabel.iter() {
        let Some(trigger_files) = value.get("trigger_files") else {
            continue
        };
        let trigger_files = trigger_files.as_array().expect("an array");
        let missing_files: Vec<_> = trigger_files
            .iter()
            // Hey TOML content is strict UTF-8 so meh.
            .map(|v| v.as_str().unwrap())
            .filter(|f| {
                // triagebot checks with `starts_with` only.
                // See https://github.com/rust-lang/triagebot/blob/add83c3ad979cee9e6120a086e36a37b5ff9edfd/src/handlers/autolabel.rs#L45
                let path = Path::new(f);
                if path.exists() {
                    return false;
                }
                let Some(mut read_dir) = path.parent().and_then(|p| p.read_dir().ok()) else {
                    return true;
                };
                !read_dir.any(|e| e.unwrap().path().to_str().unwrap().starts_with(f))
            })
            .collect();

        failed += missing_files.len();
        passed += trigger_files.len() - missing_files.len();

        if missing_files.is_empty() {
            continue;
        }

        use std::fmt::Write as _;
        let mut msg = String::new();
        writeln!(
            &mut msg,
            "missing files defined in `autolabel.{label}.trigger_files`:"
        )
        .unwrap();
        for f in missing_files.iter() {
            writeln!(&mut msg, "\t {f}").unwrap()
        }
        eprintln!("{msg}");
    }

    let result = if failed == 0 { "ok" } else { "FAILED" };
    eprintln!("test result: {result}. {passed} passed; {failed} failed;");
}
