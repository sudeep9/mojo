
mod iget;
mod iview;
mod state;
mod commit;
mod buckets;

use anyhow::Error;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    kvpath: std::path::PathBuf,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// View index value for a key
    #[clap(name="iget")]
    IndexGet{
        #[clap(value_parser)]
        bucket: String,

        #[clap(value_parser)]
        ver: u32,

        #[clap(value_parser)]
        key: u32,
    },
    /// View index value for a key
    #[clap(name="iview")]
    IndexView{
        #[clap(value_parser)]
        bucket: String,

        #[clap(value_parser)]
        ver: u32,

        #[clap(short, action)]
        additional: bool,

        #[clap(short, action)]
        keys: bool
    },
    /// View the current kv state
    #[clap(name="state")]
    State{
        /// Print additional internal numbers
        #[clap(short, action)]
        additional: bool
    },
    /// Commit the store
    #[clap(name="commit")]
    Commit{
    },
    /// List buckets
    #[clap(name="buckets")]
    Buckets{

        /// Version of the bmap
        #[clap(value_parser)]
        ver: u32,
    },
}

fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd

    match &cli.command {
        Commands::IndexGet{bucket, ver, key}  => {
            iget::cmd(&cli.kvpath, bucket.as_str(), *ver, *key)?;
        },
        Commands::IndexView{bucket, ver, additional, keys}  => {
            iview::cmd(&cli.kvpath, bucket.as_str(), *ver, *additional, *keys)?;
        },
        Commands::State{additional} => {
            state::cmd(&cli.kvpath, *additional)?;
        },

        Commands::Commit{} => {
            commit::cmd(&cli.kvpath)?;
        },
        Commands::Buckets{ver} => {
            buckets::cmd(&cli.kvpath, *ver)?;
        },
    }

    Ok(())
}
