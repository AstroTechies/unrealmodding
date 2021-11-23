use clap::{Parser, Subcommand};

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
        pakfile: String
    },
    CheckHeader {
        /// The .pak file to check
        pakfile: String
    },
    Extract {
        /// The .pak file to extract
        pakfile: String,
        /// The directory to extract to
        outdir: Option<String>
    },
    Create {
        /// The .pak file to create
        pakfile: String,
        /// The directory to create the file from
        indir: String
    },
}

fn main() {
    let args = Args::parse();

    println!("{:?}", args);
}
