#![recursion_limit = "1024"]

#[macro_use]
extern crate clap;
#[macro_use]
extern crate error_chain;
extern crate mondrian_schema_cat;
extern crate walkdir;

use clap::{App, Arg, AppSettings};
use mondrian_schema_cat::fragments_to_schema;
use std::io::{Read, Write, BufWriter};
use std::fs::{self, File};
use walkdir::{DirEntry, WalkDir};

mod error {
    use mondrian_schema_cat;
    use walkdir;

    error_chain! {
        foreign_links {
            Io(::std::io::Error);
            WalkDir(walkdir::Error);
        }

        links {
            MonCat(
                mondrian_schema_cat::error::Error,
                mondrian_schema_cat::error::ErrorKind
            );
        }
    }
}

use error ::*;

fn main() {
    if let Err(ref err) = run() {
        println!("error: {}", err);

        for e in err.iter().skip(1) {
            println!(" cause by: {}", e);
        }

        if let Some(backtrace) = err.backtrace() {
            println!("backtrace: {:?}", backtrace);
        }

        ::std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let config = get_cli_config();

    let fragment_paths;
    if let Some(dir_path) = config.dir_path {
        fragment_paths = get_fragment_paths_dir(&dir_path)?;
    } else {
        fragment_paths = config.arg_files;
    }

    if fragment_paths.is_empty() {
        return Err("No files found".into());
    }

    let mut fragment_strs = Vec::new();

    for file_path in fragment_paths {
        let mut f = File::open(file_path)?;
        let mut buf = String::new();

        f.read_to_string(&mut buf)?;
        fragment_strs.push(buf);
    }

    let res = fragments_to_schema(fragment_strs.as_slice())?;

    match config.output_path {
        Some(path) => {
            let f = File::create(&path)?;
            write(f, &res)?;
        },
        None => {
            write(std::io::stdout(), &res)?;
        }
    }
    Ok(())
}

fn get_fragment_paths_dir(dir_path: &str) -> Result<Vec<String>> {
    fn is_hidden(entry: &DirEntry) -> bool {
        entry.file_name()
            .to_str()
            .map(|s| s.starts_with("."))
            .unwrap_or(false)
    }

    fn is_xml(entry: &DirEntry) -> bool {
        if entry.file_type().is_file() {
            entry.file_name()
                .to_str()
                .map(|s| s.ends_with(".xml"))
                .unwrap_or(false)
        } else {
            // this is for directories
            true
        }
    }

    if !fs::metadata(dir_path)?.is_dir() {
        return Err("Path is not a directory".into());
    }

    let mut res = Vec::new();
    let walker = WalkDir::new(dir_path).into_iter();
    for entry in walker.filter_entry(|e| (!is_hidden(e)) && is_xml(e)) {
        let entry = entry?;
        if entry.file_type().is_file() {
            res.push(entry.path().to_str().expect("filepath is invalid str").to_owned())
        }
    }

    Ok(res)
}

struct Config {
    arg_files: Vec<String>,
    dir_path: Option<String>,
    output_path: Option<String>,
}

fn get_cli_config() -> Config {
    let app_m = App::new("moncat")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .setting(AppSettings::ArgRequiredElseHelp)
        .arg(Arg::with_name("arg_files")
            .takes_value(true)
            .value_name("PATH")
            .multiple(true)
            .help("file paths to fragments. Specify multiple"))
        .arg(Arg::with_name("dir_path")
            .short("d")
            .long("dir")
            .takes_value(true)
            .value_name("PATH")
            .conflicts_with("arg_files")
            .help("optional dir path, exclusive of files from args"))
        .arg(Arg::with_name("output_path")
            .short("o")
            .long("output")
            .takes_value(true)
            .value_name("PATH")
            .help("optional output path, otherwise stdout"))
        .after_help("ABOUT:\n\
            \tA utility for concatenating together fragments of a Mondrian schema.\n\
            \n\
            \tTakes an arbitrary number of schema fragments containing:\n\
            \t- schema (containing cubes and shared dims)\n\
            \t- shared dims\n\
            \t- cubes\n\
            \n\
            \tand then concatenates the fragement sections in the correct\n\
            \torder (schema wraps shared dims and then cubes, in that order).\n\
            \n\
            \tFragments can be any of the above three in any combination, but\n\
            \teach fragment's internals must be in the same order as a full schema.")
        .get_matches();

    let arg_files = app_m.values_of("arg_files");
    let arg_files = if arg_files.is_some() {
        arg_files.unwrap().map(|s| s.to_owned()).collect()
    } else {
        Vec::new()
    };

     Config {
         arg_files: arg_files,
         dir_path: app_m.value_of("dir_path").map(|s| s.to_owned()),
         output_path: app_m.value_of("output_path").map(|s| s.to_owned()),
     }
}

fn write<W: Write>(wtr: W, schema: &str) -> Result<()> {
    let mut wtr = BufWriter::new(wtr);

    wtr.write_all(schema.as_bytes())?;
    wtr.flush()?;
    Ok(())
}
