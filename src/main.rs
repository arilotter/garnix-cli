use clap::Parser;
use garnix_cli::{
    Result,
    cli::{self, Cli, Commands},
    config, git,
    matcher::AttributeMatcher,
    nix::NixFlake,
};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { as_branch, dry_run } => {
            run_build(as_branch, dry_run).await?;
        }
    }

    Ok(())
}

async fn run_build(as_branch: Option<String>, dry_run: bool) -> Result<()> {
    let current_branch = git::get_branch_or_override(as_branch)?;
    cli::print_success(&format!("running builds for branch: {}", current_branch));

    let git_root = git::get_git_root()?;
    let config = config::load_config_from_git_root(&git_root)?;

    cli::print_success(if config.is_some() {
        "config loaded from garnix.yaml"
    } else {
        "no garnix config found, using defaults"
    });

    let flake = NixFlake::from_git_root(&git_root)?;
    let available_attrs = flake.discover_attributes().await?;
    let matcher = AttributeMatcher::new(current_branch);
    let matching_attrs = matcher.get_matching_attributes(&config, &available_attrs)?;

    if matching_attrs.is_empty() {
        cli::print_warning("no attributes match the current config");
        println!();
        cli::print_info("available attributes:");
        for attr in &available_attrs {
            println!("    {}", attr);
        }
        return Ok(());
    }

    cli::print_success(&format!(
        "matched {}/{} attributes for building:",
        matching_attrs.len(),
        available_attrs.len()
    ));
    for attr in &matching_attrs {
        cli::print_build_target(attr);
    }

    match flake.build_attributes(&matching_attrs, dry_run).await {
        Ok(()) => {
            println!();
            cli::print_success("all builds completed");
        }
        Err(e) => {
            println!();
            cli::print_error(&format!("build failed: {}", e));
            std::process::exit(1);
        }
    }

    Ok(())
}
