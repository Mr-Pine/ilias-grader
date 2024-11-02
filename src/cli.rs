use std::path::PathBuf;

use clap::{Parser, Subcommand};
use regex::Regex;

#[derive(Debug, Parser)]
#[command(arg_required_else_help = true)]
pub struct Cli {
    /// The id of the exercise you want to handle
    #[arg(long, short)]
    pub id: u32,

    /// The index of the assignment you want to handle
    #[arg(long, short)]
    pub assignment: usize,

    #[arg(short, long)]
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
        feedback_dir: PathBuf,

        /// A regular expression to filter feedback files
        #[arg(long, short)]
        filter_expr: Option<Regex>,

        /// A suffix to append to uploaded feedback files
        #[arg(long, short)]
        suffix: Option<String>,

        /// Upload without confirmation
        #[arg(long)]
        no_confim: bool,
    },
}
