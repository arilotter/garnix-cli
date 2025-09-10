use clap::{Parser, Subcommand};
use colored::*;

#[derive(Parser)]
#[command(name = "garnix")]
#[command(version = "0.1.0")]
#[command(about = "CLI tooling for garnix")]
#[command(long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run builds according to garnix.yaml configuration
    Run {
        /// Override the current git branch for configuration filtering
        #[arg(long, value_name = "BRANCH")]
        as_branch: Option<String>,

        /// Don't actually run builds, just output the list of builds that *would* have run
        #[arg(long, action)]
        dry_run: bool,
    },
}

pub fn print_success(message: &str) {
    println!("{}", message.green());
}

pub fn print_warning(message: &str) {
    println!("{}", message.yellow());
}

pub fn print_error(message: &str) {
    println!("{}", message.red());
}

pub fn print_info(message: &str) {
    println!("{}", message.blue());
}

pub fn print_build_target(target: &str) {
    println!("    {}", target.cyan());
}
