use anyhow::Result;
use clap::{Arg, Command};
use dirs::home_dir;
use git2::Repository;
use std::os::windows::fs::{symlink_dir, symlink_file};
use std::path::PathBuf;
use std::process::exit;
use std::{fs, io, path::Path};

fn main() -> Result<()> {
    // Setup CLI with clap
    let matches = Command::new("Igor CLI")
        .version("0.1")
        .author("Your Name <your.email@example.com>")
        .about("Installs Igor Pro procedure files")
        .subcommand(
            Command::new("install")
                .about("Install procedure files")
                .arg(
                    Arg::new("git")
                        .short('g')
                        .long("git")
                        .num_args(1)
                        .help("GitHub repository to install from"),
                )
                .arg(
                    Arg::new("path")
                        .short('p')
                        .long("path")
                        .num_args(1)
                        .help("Local directory to install from"),
                )
                .arg(
                    Arg::new("version")
                        .short('v')
                        .long("version")
                        .num_args(1)
                        .help("Specify the Igor Pro version"),
                ),
        )
        .subcommand(Command::new("versions").about("List available Igor Pro versions"))
        .get_matches();

    // Handle the 'install' command
    if let Some(matches) = matches.subcommand_matches("install") {
        let repo_path = matches
            .get_one::<String>("git")
            .or(matches.get_one::<String>("path"));
        let version = matches.get_one::<String>("version");

        if let Some(repo_path) = repo_path {
            install_procedure_files(repo_path, version.map(|s| s.as_str()))?;
        } else {
            println!("Please provide a valid path or GitHub repository");
            exit(1);
        }
    }

    // Handle the 'versions' command
    if matches.subcommand_matches("versions").is_some() {
        list_igor_versions()?;
    }

    Ok(())
}

// Install procedure files from Git or local path
fn install_procedure_files(repo_path: &str, version: Option<&str>) -> Result<()> {
    // Determine the path to the .igor directory in the user's home folder
    let repo_dir = if is_git_url(repo_path) {
        clone_repository_into_igor(repo_path)?
    } else {
        Path::new(repo_path).to_path_buf()
    };

    // Verify folder structure
    let user_dir = repo_dir.join("user");
    let igor_dir = repo_dir.join("igor");

    if !user_dir.exists() || !igor_dir.exists() {
        println!("The required 'user' and 'igor' directories are missing. Please modify the repository structure.");
        exit(1);
    }

    let igor_version = match version {
        Some(v) => v.to_string(),
        None => select_igor_version()?, // If no version is specified, prompt the user
    };

    // Get Igor Pro paths for User Procedures and Igor Procedures
    let user_procs = get_wave_metrics_path(&igor_version, "User Procedures")?;
    let igor_procs = get_wave_metrics_path(&igor_version, "Igor Procedures")?;

    // Create symbolic links for user and igor procedure files
    link_files(&user_dir, &user_procs)?;
    link_files(&igor_dir, &igor_procs)?;

    println!(
        "Successfully installed procedures for Igor Pro {}",
        igor_version
    );
    Ok(())
}

// Clone repository into the user's $HOME/.igor folder
fn clone_repository_into_igor(repo: &str) -> Result<PathBuf> {
    // Get the user's home directory and append ".igor"
    let home_dir = home_dir().expect("Could not find the user's home directory.");
    let igor_dir = home_dir.join(".igor");

    // Ensure the .igor directory exists
    if !igor_dir.exists() {
        fs::create_dir_all(&igor_dir)?;
        println!("Created .igor directory at: {:?}", igor_dir);
    }

    // Determine the repo directory under .igor based on repo name
    let repo_name = repo.split('/').last().unwrap_or("repository"); // Get the last part of the repo URL
    let repo_dir = igor_dir.join(repo_name);

    // Clone the repository into this path
    if !repo_dir.exists() {
        println!("Cloning repository into {:?}", repo_dir);
        Repository::clone(repo, &repo_dir)?;
    } else {
        println!("Repository already exists at {:?}", repo_dir);
    }

    Ok(repo_dir)
}

// Helper to detect if a URL is a GitHub URL
fn is_git_url(repo: &str) -> bool {
    repo.starts_with("http") || repo.starts_with("git")
}

// List available Igor Pro versions
fn list_igor_versions() -> Result<()> {
    let igor_dir = Path::new("C:\\Program Files\\WaveMetrics");
    let versions: Vec<_> = fs::read_dir(igor_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect();

    println!("Available Igor Pro Versions:");
    for version in versions {
        println!("{}", version);
    }
    Ok(())
}

// Select Igor version interactively
fn select_igor_version() -> Result<String> {
    list_igor_versions()?;
    println!("Please enter the desired version:");
    let mut version = String::new();
    io::stdin().read_line(&mut version)?;
    Ok(version.trim().to_string())
}

// Get WaveMetrics path based on version and folder type (User Procedures or Igor Procedures)
fn get_wave_metrics_path(version: &str, folder_type: &str) -> Result<PathBuf> {
    let doc_dir = dirs::document_dir().expect("Could not locate the user's Documents folder.");
    let path = doc_dir.join(format!(
        "WaveMetrics/Igor Pro {} User Files/{}",
        version, folder_type
    ));
    Ok(path)
}

// Helper to link files and directories
fn link_files(src_dir: &Path, dst_dir: &Path) -> Result<()> {
    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst_dir.join(entry.file_name());

        if src_path.is_file() {
            symlink_file(&src_path, &dst_path)?;
        } else if src_path.is_dir() {
            symlink_dir(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
