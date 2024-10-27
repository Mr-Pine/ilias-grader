use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(arg_required_else_help = true)]
pub struct Cli {
    /// The id of the exercise you want to handle
    #[arg(long, short, required = true)]
    pub id: u32,

    /// The index of the assignment you want to handle
    #[arg(long, short, required = true)]
    pub assignment: usize,

    #[arg(short, long, required = true)]
    pub username: String,

    #[arg(short, long)]
    pub password: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Download submissions
    Download {
        /// The path to download the assignments to
        #[arg(required = true)]
        to: PathBuf,

        /// Whether to extract the zip file
        #[arg(long)]
        extract: bool,

        /// Whether to flatten the extracted files into one directory
        #[arg(long)]
        flatten: bool,
    },
    /// Upload feedback
    Feedback {
        /// The directory where your feedback files are located
        #[arg(required = true)]
        feedback_dir: PathBuf,

        /// A suffix to append to uploaded feedback files
        #[arg(long, short, default_value = "")]
        suffix: String,

        /// Upload without confirmation
        #[arg(long, default_value = "false")]
        no_confim: bool,
    },
}
