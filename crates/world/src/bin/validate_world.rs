use std::path::PathBuf;

use world::validator::{validate_project, validate_project_and_quarantine};

fn main() -> anyhow::Result<()> {
    let mut json = false;
    let mut quarantine = false;
    let mut path: Option<PathBuf> = None;

    let args = std::env::args().skip(1);
    for arg in args {
        match arg.as_str() {
            "--json" => json = true,
            "--quarantine" => quarantine = true,
            value => {
                if path.is_some() {
                    return Err(anyhow::anyhow!("unexpected argument: {}", value));
                }
                path = Some(PathBuf::from(value));
            }
        }
    }

    let project_root = path.unwrap_or_else(|| PathBuf::from("."));
    let issues = if quarantine {
        validate_project_and_quarantine(&project_root)
    } else {
        validate_project(&project_root)
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&issues)?);
    } else {
        for issue in &issues {
            if let Some(path) = &issue.path {
                println!("{}: {}", path.display(), issue.message);
            } else {
                println!("{}", issue.message);
            }
        }
    }

    if issues.is_empty() {
        Ok(())
    } else {
        std::process::exit(1);
    }
}
