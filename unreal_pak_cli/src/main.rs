use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::process::exit;
use std::time::SystemTime;

use clap::{Parser, Subcommand};
use unreal_pak::PakRecord;
use walkdir::WalkDir;

/// Command line tool for working with Unreal Engine .pak files.
/// Use `unreal_pak_cli <SUBCOMMAND> -h` for more information on a subcommand.
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
    /// Check an entire .pak file if it is valid.
    Check {
        /// The .pak file to check
        pakfile: String,
    },

    /// Only check the header of a .pak file if it is valid.
    CheckHeader {
        /// The .pak file to check
        pakfile: String,
    },

    /// Extract a .pak file to a directory.
    Extract {
        /// The .pak file to extract
        pakfile: String,
        /// The directory to extract to, if not specified the .pak file name will be used
        outdir: Option<String>,
    },

    /// create a new .pak file from the files from a directory, optionally disabling compression.
    Create {
        /// The .pak file to create
        pakfile: String,
        /// The directory to create the file from
        indir: String,
        /// Whether to compress the file
        #[clap(short, long)]
        no_compression: bool,
    },
}

fn main() {
    let args = Args::parse();

    let start = SystemTime::now();

    match args.commands {
        Commands::CheckHeader { pakfile } => {
            let file = open_file(Path::new(&pakfile));
            let mut pak = unreal_pak::PakFile::reader(&file);
            check_header(&mut pak);
        }
        Commands::Check { pakfile } => {
            let file = open_file(Path::new(&pakfile));
            let mut pak = unreal_pak::PakFile::reader(&file);
            check_header(&mut pak);

            // TODO: get rid of this clone
            for (i, (record_name, _)) in pak.records.clone().iter().enumerate() {
                println!("Record {}: {}", i, record_name);

                match pak.get_record(record_name) {
                    Ok(_) => (),
                    Err(e) => {
                        eprintln!("Error reading record {}: {}, Error: {}", i, record_name, e);
                        exit(1);
                    }
                }
            }
        }
        Commands::Extract { pakfile, outdir } => {
            let path = Path::new(&pakfile);
            let file = open_file(path);
            let mut pak = unreal_pak::PakFile::reader(&file);
            check_header(&mut pak);

            // temp values required to extend lifetimes outside of match scope
            let temp;
            let temp2;
            let output_folder: &Path = match outdir {
                Some(ref outdir) => {
                    temp = outdir.clone();
                    Path::new(&temp)
                }
                None => {
                    temp2 = path.parent().unwrap().join(path.file_stem().unwrap());
                    &temp2
                }
            };

            println!("Extracting to {}", output_folder.display());

            // TODO: get rid of this clone
            for (i, (record_name, _)) in pak.records.clone().iter().enumerate() {
                match pak.get_record(record_name) {
                    Ok(record_data) => {
                        let path = Path::new(output_folder).join(&record_name[..]);
                        let dir_path = match path.parent() {
                            Some(dir) => dir,
                            None => {
                                eprintln!("No parent directories found! {}: {}", i, record_name);
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
                                        eprintln!("Error creating file! {}: {:?}", i, path);
                                        exit(1);
                                    }
                                };
                                // Write the file
                                match file.write_all(record_data.data.as_ref().unwrap()) {
                                    Ok(_) => {
                                        println!("Record {}: {}", i, record_name);
                                    }
                                    Err(_) => {
                                        eprintln!("Error writing to file! {}: {:?}", i, path);
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
                        eprintln!("Error reading record {}: {}", i, record_name);
                        exit(1);
                    }
                }
            }
        }
        Commands::Create {
            pakfile,
            indir,
            no_compression,
        } => {
            // clear file
            OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&pakfile)
                .unwrap();

            let file = OpenOptions::new().append(true).open(&pakfile).unwrap();

            let mut pak = unreal_pak::PakFile::writer(
                unreal_pak::pakversion::PakVersion::PakFileVersionFnameBasedCompressionMethod,
                &file,
            );

            let compression_method = if no_compression {
                unreal_pak::CompressionMethod::None
            } else {
                unreal_pak::CompressionMethod::Zlib
            };

            println!("Using compression method: {:?}", compression_method);

            // Get all files and write them to the .pak file
            for entry in WalkDir::new(&indir) {
                let entry = entry.unwrap();
                if entry.file_type().is_file() {
                    let file_path = entry.path().to_str().unwrap().to_owned();

                    let mut record_name = file_path[indir.len()..].to_owned();
                    if record_name.starts_with('/') {
                        record_name = record_name[1..].to_owned();
                    }

                    println!("Adding record: {}", record_name);

                    let file_data = match std::fs::read(&file_path) {
                        Ok(file_data) => file_data,
                        Err(_) => {
                            eprintln!("Error reading file! {}", file_path);
                            exit(1);
                        }
                    };

                    let record = PakRecord::new(record_name.clone(), file_data, compression_method)
                        .unwrap_or_else(|_| {
                            panic!("Error creating record {}", record_name.clone())
                        });
                    pak.add_record(record)
                        .unwrap_or_else(|_| panic!("Error adding record {}", record_name));
                }
            }

            pak.write().expect("Failed to write");
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

fn check_header(pak: &mut unreal_pak::PakFile) {
    match pak.load_records() {
        Ok(_) => println!("Header is ok"),
        Err(e) => {
            eprintln!("Error reading header: {}", e);
            exit(1);
        }
    }
    println!("Found {:?} records", pak.records.len());
}
