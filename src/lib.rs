#![deny(clippy::pedantic, clippy::style)]
use std::{
    error::Error,
    fs::{read_to_string, FileType},
    path::PathBuf,
};

type RibbitR<T> = Result<T, Box<dyn Error>>;

use clap::Parser;

#[derive(Parser, Debug)]
struct Cli {
    #[arg(required = true)]
    journal: PathBuf,
}

pub fn run() {
    let matches = Cli::parse();
    let mut md_files = Vec::new();
    find_files(&mut md_files, matches.journal).unwrap();

    let front_matters: Vec<Vec<_>> = md_files
        .iter()
        .filter_map(|f| {
            let file = read_to_string(f).unwrap();
            let mut file = file.lines();
            let mut fm = Vec::new();

            match (
                file.next(),
                file.next(),
                file.next(),
                file.next(),
                file.next(),
                file.next(),
                file.next(),
            ) {
                (Some("---"), Some(date), Some(h), Some(w), Some(c), Some(r), Some("---")) => {
                    fm.push(date.to_owned());
                    fm.push(h.to_owned());
                    fm.push(w.to_owned());
                    fm.push(c.to_owned());
                    fm.push(r.to_owned());
                    Some(fm)
                }
                _ => None,
            }
        })
        .collect();
    dbg!(md_files);
    dbg!(front_matters);
}

fn find_files(md_files: &mut Vec<PathBuf>, dir: PathBuf) -> RibbitR<()> {
    for el in dir.read_dir()? {
        let path = el.as_ref().unwrap().path();
        if let Ok(ft) = el?.file_type() {
            if ft.is_dir() {
                find_files(md_files, path.to_path_buf())?
            } else {
                if let Some(ext) = path.extension() {
                    if ext == "md" {
                        md_files.push(path.to_path_buf());
                    }
                }
            }
        }
    }
    Ok(())
}
