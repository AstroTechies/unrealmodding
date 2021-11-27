use std::fs::File;
use std::fs::OpenOptions;
use std::path::Path;
use std::io::Write;
use std::process::exit;
use std::time::SystemTime;

use clap::{Parser, Subcommand};

use upak;

/// Command line tool for working with Unreal Engine .pak files
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Args {
    /// What to do
    #[clap(subcommand)]
    commands: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Adds files to myapp
    Check {
        /// The .pak file to check
        pakfile: String,
    },
    CheckHeader {
        /// The .pak file to check
        pakfile: String,
    },
    Extract {
        /// The .pak file to extract
        pakfile: String,
        /// The directory to extract to
        outdir: Option<String>,
    },
    Create {
        /// The .pak file to create
        pakfile: String,
        /// The directory to create the file from
        indir: String,
    },
}

fn main() {
    let args = Args::parse();

    let start = SystemTime::now();

    match args.commands {
        Commands::CheckHeader { pakfile } => {
            let file = open_file(Path::new(&pakfile));
            let mut pak = upak::PakFile::new(&file);
            check_header(&mut pak);
        }
        Commands::Check { pakfile } => {
            let file = open_file(Path::new(&pakfile));
            let mut pak = upak::PakFile::new(&file);
            check_header(&mut pak);

            // TODO: get rid of this clone
            for (i, (record_name, _)) in pak.records.clone().iter().enumerate() {
                println!("Record {}: {}", i.to_string(), record_name);

                match pak.read_record(&record_name) {
                    Ok(_) => (),
                    Err(e) => {
                        eprintln!(
                            "Error reading record {}: {}, Error: {}",
                            i.to_string(),
                            record_name,
                            e
                        );
                        exit(1);
                    }
                }
            }
        }
        Commands::Extract { pakfile, outdir } => {
            let path = Path::new(&pakfile);
            let file = open_file(&path);
            let mut pak = upak::PakFile::new(&file);
            check_header(&mut pak);

            // temp values required to extend lifetimes outside of match scope
            let temp;
            let temp2;
            let output_folder: &Path = match outdir {
                Some(ref outdir) => {
                    temp = outdir.clone();
                    Path::new(&temp)},
                None => {
                    temp2 = path.parent().unwrap().join(path.file_stem().unwrap());
                    &temp2
                },
            };

            println!("Extracting to {}", output_folder.display());

            // TODO: get rid of this clone
            for (i, (record_name, _)) in pak.records.clone().iter().enumerate() {
                match pak.read_record(&record_name) {
                    Ok(record_data) => {
                        let path = Path::new(output_folder).join(&record_name);
                        let dir_path = match path.parent() {
                            Some(dir) => dir,
                            None => {
                                eprintln!(
                                    "No parent directories found! {}: {}",
                                    i.to_string(),
                                    record_name
                                );
                                exit(1);
                            }
                        };

                        // Create the parent directories, then files.
                        match std::fs::create_dir_all(dir_path) {
                            Ok(_) => {
                                // Create the file
                                let mut file = match File::create(&path) {
                                    Ok(file) => file,
                                    Err(_) => {
                                        eprintln!(
                                            "Error creating file! {}: {:?}",
                                            i.to_string(),
                                            path
                                        );
                                        exit(1);
                                    }
                                };
                                // Write the file
                                match file.write_all(&record_data) {
                                    Ok(_) => {
                                        println!("Record {}: {}", i.to_string(), record_name);
                                    }
                                    Err(_) => {
                                        eprintln!(
                                            "Error writing to file! {}: {:?}",
                                            i.to_string(),
                                            path
                                        );
                                        exit(1);
                                    }
                                }
                            }
                            Err(_) => {
                                eprintln!("Error creating directories! {:?}", dir_path);
                                exit(1);
                            }
                        };
                    }
                    Err(_) => {
                        eprintln!("Error reading record {}: {}", i.to_string(), record_name);
                        exit(1);
                    }
                }
            }
        }
        _ => {
            eprintln!("Not implemented yet");
        }
    }

    println!(
        "upakcli took {:?} seconds...",
        start.elapsed().unwrap().as_secs_f32()
    )
}

fn open_file(path: &Path) -> File {
    match OpenOptions::new().read(true).open(&path) {
        Ok(file) => file,
        Err(_) => {
            eprintln!("Could not find/open file");
            exit(1);
        }
    }
}

fn check_header(pak_file: &mut upak::PakFile) {
    match pak_file.load_records() {
        Ok(_) => println!("Header is ok"),
        Err(e) => {
            eprintln!("Error reading header: {}", e);
            exit(1);
        }
    }
    println!("Found {:?} records", pak_file.records.len());
}
