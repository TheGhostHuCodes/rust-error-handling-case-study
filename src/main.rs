use docopt::Docopt;
use serde::Deserialize;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io;
use std::path::Path;
use std::process;

macro_rules! fatal {
    ($($tt:tt)*) => {{
        use std::io::Write;
        writeln!(&mut ::std::io::stderr(), $($tt)*).unwrap();
        ::std::process::exit(1)
    }}
}

static USAGE: &'static str = "
Usage: city-pop [options] [<data-path>] <city>
       city-pop --help

Options:
    -h, --help          Show this usage message.
    -q, --quiet         Do not show noisy messages.
    -u, --show-unknown  Show cities with unknown population.
";

#[derive(Deserialize, Debug)]
struct Args {
    arg_data_path: Option<String>,
    arg_city: String,
    flag_quiet: bool,
    flag_show_unknown: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct PopulationCount {
    city: String,
    country: String,
    region: String,
    population: Option<u64>,
}

#[derive(Debug)]
enum CliError {
    Io(io::Error),
    Csv(csv::Error),
    NotFound,
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CliError::Io(ref err) => err.fmt(f),
            CliError::Csv(ref err) => err.fmt(f),
            CliError::NotFound => write!(f, "No matching cities with a population were found."),
        }
    }
}

impl Error for CliError {}

impl From<io::Error> for CliError {
    fn from(err: io::Error) -> CliError {
        CliError::Io(err)
    }
}

impl From<csv::Error> for CliError {
    fn from(err: csv::Error) -> CliError {
        CliError::Csv(err)
    }
}

fn search<P: AsRef<Path>>(
    file_path: &Option<P>,
    city: &str,
    include_unknowns: &bool,
) -> Result<Vec<PopulationCount>, CliError> {
    let mut found = vec![];
    let input: Box<dyn io::Read> = match *file_path {
        None => Box::new(io::stdin()),
        Some(ref file_path) => Box::new(File::open(file_path)?),
    };
    let mut rdr = csv::Reader::from_reader(input);
    for row in rdr.deserialize() {
        let row: PopulationCount = row?;
        if *include_unknowns && row.city == city {
            found.push(row)
        } else {
            match row.population {
                None => {} // Skip it.
                Some(_) => {
                    if row.city == city {
                        found.push(row)
                    }
                }
            }
        }
    }

    if found.is_empty() {
        Err(CliError::NotFound)
    } else {
        Ok(found)
    }
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|err| err.exit());

    match search(&args.arg_data_path, &args.arg_city, &args.flag_show_unknown) {
        Err(CliError::NotFound) if args.flag_quiet => process::exit(1),
        Err(err) => fatal!("{}", err),
        Ok(populated) => {
            for p in populated {
                println!(
                    "{}, {}, {}: {}",
                    p.city,
                    p.region,
                    p.country,
                    p.population
                        .map(|i| i.to_string())
                        .unwrap_or("Unknown Population".to_owned())
                );
            }
        }
    }
}
