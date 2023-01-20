use std::{error::Error, fs::read_to_string, ops::Add, path::PathBuf};

type RibbitR<T> = Result<T, Box<dyn Error>>;

use chrono::{Datelike, Local};
use clap::{Parser, ValueEnum};
use serde::Deserialize;

#[derive(Parser, Debug)]
struct Cli {
    #[arg(default_value = "/Users/z/journal")]
    journal_dir: PathBuf,

    #[command(subcommand)]
    action: Option<Action>,
}

#[derive(clap::Subcommand, Debug)]
enum Action {
    Filter {
        #[arg(value_enum)]
        habit: Option<HabitFilter>,

        #[arg(short, long, value_enum)]
        time: Option<TimeFilter>,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum HabitFilter {
    Exercise,
    Contrib,
    Reading,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum TimeFilter {
    #[value(alias("w"))]
    Week,
    #[value(alias("m"))]
    Month,
    #[value(alias("d"))]
    Day,
    #[value(alias("y"))]
    Year,
}

#[derive(Deserialize, Debug, Clone, Copy)]
struct Habit {
    exercise: bool,
    contrib: bool,
    reading: bool,
}

#[derive(Deserialize)]
struct Fm {
    title: String,
    date: chrono::NaiveDate,
    habits: Habit,
}

fn filter_by_time(fm: Vec<Fm>, filter: TimeFilter) -> Vec<Fm> {
    let today = Local::now().date_naive();
    fm.into_iter()
        .filter(|f| match filter {
            TimeFilter::Day => f.date.eq(&today),
            TimeFilter::Week => f.date.iso_week().eq(&today.iso_week()),
            TimeFilter::Month => f.date.month().eq(&today.month()),
            TimeFilter::Year => f.date.year().eq(&today.year()),
        })
        .collect()
}

fn count(fm: Vec<Fm>) -> HabitCount {
    fm.into_iter()
        .take(7)
        .fold(HabitCount::default(), |mut count: HabitCount, fm| {
            count = count + fm.habits;
            count
        })
}
fn count_filtered_habit(fm: Vec<Fm>, filter: HabitFilter) -> HabitCount {
    let fm: Vec<Fm> = fm
        .into_iter()
        .filter(|f| match filter {
            HabitFilter::Exercise => f.habits.exercise == true,
            HabitFilter::Contrib => f.habits.contrib == true,
            HabitFilter::Reading => f.habits.reading == true,
        })
        .collect();

    count(fm)
}

fn count_filtered_time(fm: Vec<Fm>, filter: TimeFilter) -> HabitCount {
    count(filter_by_time(fm, filter))
}

pub fn run() -> RibbitR<()> {
    let matches = Cli::parse();
    let mut files = Vec::new();
    find_files(&mut files, matches.journal_dir)?;

    let mut front_matters = parse_frontmatter(files)?;
    front_matters.sort_by_key(|f| f.date);

    match matches.action {
        Some(Action::Filter {
            habit: filter,
            time,
        }) => match (filter, time) {
            (Some(filter), None) => {
                let count = count_filtered_habit(front_matters, filter);
                count.print_filtered(filter);
            }
            (None, None) => {
                let count = count(front_matters);
                count.print();
            }
            (None, Some(time)) => {
                let count = count_filtered_time(front_matters, time);
                count.print();
            }
            (Some(filter), Some(time)) => {
                let fm = filter_by_time(front_matters, time);
                let count = count_filtered_habit(fm, filter);
                count.print_filtered(filter);
            }
        },
        None => {
            let count = count(front_matters);
            count.print();
        }
    }

    Ok(())
}

fn parse_frontmatter(md_files: Vec<PathBuf>) -> RibbitR<Vec<Fm>> {
    Ok(md_files
        .iter()
        .filter_map(|f| {
            if let Ok(file) = read_to_string(f) {
                let file = file.lines();
                let mut fm = Vec::new();

                let mut delim = false;
                for line in file {
                    if line == "---" {
                        delim = !delim;
                    } else {
                        if delim {
                            fm.push(line.to_owned());
                        }
                    }
                }
                if fm.len() > 0 {
                    Some(fm)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .filter_map(|fm| serde_yaml::from_str::<Fm>(fm.join("\n").as_str()).ok())
        .collect())
}

#[derive(Default, Debug, Clone, Copy)]
struct HabitCount {
    exercise: usize,
    reading: usize,
    contrib: usize,
}
impl Add<Habit> for HabitCount {
    type Output = HabitCount;

    fn add(self, rhs: Habit) -> Self::Output {
        let mut hc = self;
        if rhs.exercise {
            hc.exercise += 1;
        }
        if rhs.contrib {
            hc.contrib += 1;
        }
        if rhs.reading {
            hc.reading += 1;
        }
        hc
    }
}

impl HabitCount {
    fn print_filtered(&self, filter: HabitFilter) {
        match filter {
            HabitFilter::Exercise => println!("{:>4} - exercise", self.exercise),
            HabitFilter::Contrib => println!("{:>4} - contrib", self.contrib),
            HabitFilter::Reading => println!("{:>4} - reading", self.reading),
        }
    }
    fn print(&self) {
        println!("{:>4} - exercise", self.exercise);
        println!("{:>4} - contrib", self.contrib);
        println!("{:>4} - reading", self.reading);
    }
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
