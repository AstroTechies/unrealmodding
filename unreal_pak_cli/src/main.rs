use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::time::SystemTime;

use clap::{Parser, Subcommand};
use path_absolutize::Absolutize;
use unreal_pak::{pakversion::PakVersion, CompressionMethod, PakFile};
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
        /// The directory to create the file from
        indir: String,
        /// The .pak file to create, if not supplied the dir name will be used
        pakfile: Option<String>,
        /// Do not use compression when writing the file
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
            let mut pak = PakFile::reader(&file);
            check_header(&mut pak);
        }
        Commands::Check { pakfile } => {
            let file = open_file(Path::new(&pakfile));
            let mut pak = unreal_pak::PakFile::reader(&file);
            check_header(&mut pak);

            // TODO: get rid of this clone
            let names = pak
                .get_entry_names()
                .into_iter()
                .cloned()
                .collect::<Vec<_>>();
            for (i, file_name) in names.iter().enumerate() {
                println!("Record {}: {:?}", i, file_name);

                match pak.read_entry(file_name) {
                    Ok(_) => (),
                    Err(e) => {
                        eprintln!("Error reading record {}: {:?}! Error: {}", i, file_name, e);
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
            let output_folder: PathBuf = match outdir {
                Some(ref outdir) => PathBuf::from(outdir),
                None => path.parent().unwrap().join(path.file_stem().unwrap()),
            };

            println!("Extracting to {:?}", output_folder);

            // TODO: get rid of this clone
            let names = pak
                .get_entry_names()
                .into_iter()
                .cloned()
                .collect::<Vec<_>>();
            for (i, file_name) in names.iter().enumerate() {
                match pak.read_entry(file_name) {
                    Ok(data) => {
                        let path = output_folder.join(&file_name);
                        let dir_path = match path.parent() {
                            Some(dir) => dir,
                            None => {
                                eprintln!("No parent directories found! {}: {:?}", i, file_name);
                                exit(1);
                            }
                        };

                        // Create the parent directories, then files.
                        match std::fs::create_dir_all(dir_path) {
                            Ok(_) => {
                                // Create the file
                                let mut file = match File::create(&path) {
                                    Ok(file) => file,
                                    Err(err) => {
                                        eprintln!(
                                            "Error creating file {}: {:?}! Error: {}",
                                            i, path, err
                                        );
                                        exit(1);
                                    }
                                };
                                // Write the file
                                match file.write_all(data.as_slice()) {
                                    Ok(_) => {
                                        println!("Record {}: {}", i, file_name);
                                    }
                                    Err(err) => {
                                        eprintln!(
                                            "Error writing to file {}: {:?}! Error: {}",
                                            i, path, err
                                        );
                                        exit(1);
                                    }
                                }
                            }
                            Err(err) => {
                                eprintln!(
                                    "Error creating directories {:?}! Error: {}",
                                    dir_path, err
                                );
                                exit(1);
                            }
                        };
                    }
                    Err(err) => {
                        eprintln!(
                            "Error reading record {}: {:?}! Error: {}",
                            i, file_name, err
                        );
                        exit(1);
                    }
                }
            }
        }
        Commands::Create {
            indir,
            pakfile,
            no_compression,
        } => {
            let pakfile = match pakfile {
                Some(pakfile) => Path::new(&pakfile).absolutize().unwrap().to_path_buf(),
                None => {
                    let mut path = Path::new(&indir).absolutize().unwrap().to_path_buf();
                    path.set_extension("pak");
                    path
                }
            };
            let indir = Path::new(&indir).absolutize().unwrap().to_path_buf();
            let indir_len = indir.components().count();

            println!("Creating {:?}", pakfile);

            // clear file
            OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&pakfile)
                .unwrap();

            let file = OpenOptions::new().append(true).open(&pakfile).unwrap();

            let mut pak = unreal_pak::PakFile::writer(
                &file,
                PakVersion::PakFileVersionFnameBasedCompressionMethod,
                CompressionMethod::Zlib,
            );

            let compression_method = if no_compression {
                unreal_pak::CompressionMethod::None
            } else {
                unreal_pak::CompressionMethod::Zlib
            };

            println!("Using compression method: {:?}", compression_method);
            pak.compression = compression_method;

            // Get all files and write them to the .pak file
            let files = WalkDir::new(&indir)
                .into_iter()
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.file_type().is_file())
                .collect::<Vec<_>>();
            //files.sort_unstable_by_key(|entry| entry.file_name().to_owned());

            println!("Writing {} files", files.len());

            for (i, entry) in files.iter().enumerate() {
                // file_path is the OS absolute path, file_name is the folders and file name written to the pak
                let file_path = entry.path();
                let mut components = file_path.components();
                for _ in 0..indir_len {
                    components.next();
                }

                let mut file_name = components
                    .as_path()
                    .to_string_lossy()
                    .to_owned()
                    .replace('\\', "/");
                if file_name.starts_with('/') {
                    file_name = file_name[1..].to_owned();
                }

                let file_data = match std::fs::read(&file_path) {
                    Ok(file_data) => file_data,
                    Err(err) => {
                        eprintln!("Error reading file {:?}! Error: {}", file_path, err);
                        exit(1);
                    }
                };

                match pak.write_entry(&file_name, &file_data) {
                    Ok(_) => println!("Wrote file {}: {}", i, file_name),
                    Err(err) => {
                        eprintln!("Error writing file in pak {:?}! Error: {}", file_name, err);
                        exit(1);
                    }
                }
            }

            match pak.finish_write() {
                Ok(_) => println!("Finsihed writing pak index and footer"),
                Err(err) => {
                    eprintln!("Error writing pak index or footer! Error: {}", err);
                    exit(1);
                }
            }
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
        Err(err) => {
            eprintln!("Could not find/open file! Error: {}", err);
            exit(1);
        }
    }
}

fn check_header(pak: &mut PakFile<File, File>) {
    match pak.load_index() {
        Ok(_) => println!("Header is ok"),
        Err(err) => {
            eprintln!("Error reading header! Error: {}", err);
            exit(1);
        }
    }
    println!("Found {:?} records", pak.get_entry_names().len());
}
