use anyhow::Result;
use clap::{Arg, Command};
use dirs::home_dir;
use git2::Repository;
use std::os::windows::fs::{symlink_dir, symlink_file};
use std::path::PathBuf;
use std::process::exit;
use std::{fs, path::Path};

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
                        .num_args(1) // Updated for clap 4.x
                        .help("GitHub repository to install from"),
                )
                .arg(
                    Arg::new("path")
                        .short('p')
                        .long("path")
                        .num_args(1) // Updated for clap 4.x
                        .help("Local directory to install from"),
                ),
        )
        .get_matches();

    // Handle the 'install' command
    if let Some(matches) = matches.subcommand_matches("install") {
        let repo_path = matches
            .get_one::<String>("git")
            .or(matches.get_one::<String>("path"));

        if let Some(repo_path) = repo_path {
            install_procedure_files(repo_path)?;
        } else {
            println!("Please provide a valid path or GitHub repository");
            exit(1);
        }
    }

    Ok(())
}

// Install procedure files from Git or local path
fn install_procedure_files(repo_path: &str) -> Result<()> {
    // Determine the path to the .igor directory in the user's home folder
    let repo = if is_git_url(repo_path) {
        clone_repository_into_igor(repo_path)?
    } else {
        Path::new(repo_path).to_path_buf()
    };

    let repo_dir = repo.canonicalize()?;

    // Verify folder structure
    let user_dir = repo_dir.join("user");
    let igor_dir = repo_dir.join("igor");

    if !user_dir.exists() || !igor_dir.exists() {
        println!("The required 'user' and 'igor' directories are missing. Please modify the repository structure.");
        exit(1);
    }

    // Get the highest Igor Pro version installed
    let igor_version = find_highest_igor_version()?;

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

// Find the highest version of Igor Pro installed on the system
fn find_highest_igor_version() -> Result<String> {
    let igor_dir = Path::new("C:/Program Files/WaveMetrics");
    let versions: Vec<_> = fs::read_dir(igor_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect();

    if versions.is_empty() {
        println!("No Igor Pro installations found.");
        exit(1);
    }

    // Sort versions in descending order and pick the first (highest) one
    let highest_version = versions
        .into_iter()
        .max()
        .expect("Failed to determine the highest Igor Pro version");

    Ok(highest_version)
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

        // Create symbolic links from the repository's user/igor folder to the corresponding Igor directory
        if src_path.is_file() {
            println!(
                "Creating symbolic link from {:?} to {:?}",
                &src_path, &dst_path
            );
            symlink_file(&src_path, &dst_path)?;
        } else if src_path.is_dir() {
            println!(
                "Creating symbolic link from {:?} to {:?}",
                &src_path, &dst_path
            );
            symlink_dir(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
