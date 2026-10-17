use std::{
    process,
    env,
    fs,
    path::Path,
    cmp::Ordering,
};

mod logging;

pub fn run(config: Config) -> Result<(), Error> {
    let config_directory = config.config_directory;

    let pacman_packages = get_pacman_packages()?;
    let file_packages = get_file_packages(Path::new(&config_directory), config.recursive)?;

    let (to_install, to_uninstall) = compare_package_lists(pacman_packages.into_iter(), file_packages.into_iter());
    if config.upgrade_system {
        logln!("Updating system.");
        update()?
    }
    install(to_install)?;
    if config.list_uninstall { list_uninstall(to_uninstall.iter(), to_uninstall.len() == 0); } 
    if !config.forbid_uninstall { uninstall(to_uninstall)?; }

    Ok(())
}

fn update() -> Result<(), Error> {
    let mut pacman = process::Command::new("sudo");
    pacman.args(["pacman", "-Syu"]);

    let exit_code = pacman.status()
        .map_err(|_| Error::FailedInstall(None))?
        .code().ok_or_else(|| Error::FailedInstall(None))?;
    if exit_code != 0 {
        return Err(Error::FailedInstall(Some(exit_code)));
    }

    Ok(())
}

fn list_install<'a>(packages: impl Iterator<Item = &'a String>, empty: bool) {
    if empty {
        logln!("No packages to install.");
        return;
    }

    log!("Installing");
    for package in packages {
        print!(" {package}");
    }
    println!(".");
}

fn list_uninstall<'a>(packages: impl Iterator<Item = &'a String>, empty: bool) {
    if empty {
        logln!("No packages to uninstall.");
        return;
    }

    log!("Uninstalling");
    for package in packages {
        print!(" {package}");
    }
    println!(".");
}

fn install(packages: Vec<String>) -> Result<(), Error> {
    list_install(packages.iter(), packages.len() == 0);

    if packages.len() == 0 {
        return Ok(());
    }

    let mut pacman = process::Command::new("sudo");
    pacman.args(["pacman", "-S"]);

    for package in packages.iter() {
        pacman.arg(package);
    }

    let exit_code = pacman.status()
        .map_err(|_| Error::FailedInstall(None))?
        .code().ok_or_else(|| Error::FailedInstall(None))?;
    if exit_code != 0 {
        return Err(Error::FailedInstall(Some(exit_code)));
    }

    Ok(())
}

fn uninstall(packages: Vec<String>) -> Result<(), Error> {
    list_uninstall(packages.iter(), packages.len() == 0);

    if packages.len() == 0 {
        return Ok(());
    }

    let mut pacman = process::Command::new("sudo");
    pacman.args(["pacman", "-Rns"]);
    for package in packages.iter() {
        pacman.arg(package);
    }

    let exit_code = pacman.status()
        .map_err(|_| Error::FailedUninstall(None))?
        .code().ok_or_else(|| Error::FailedUninstall(None))?;
    if exit_code != 0 {
        return Err(Error::FailedUninstall(Some(exit_code)));
    }

    Ok(())
}

fn compare_package_lists<T>(mut pacman_packages: T, mut file_packages: T) -> (Vec<String>, Vec<String>)
    where T: Iterator<Item = String>
{
    let mut to_install: Vec<String> = Vec::new();
    let mut to_uninstall: Vec<String> = Vec::new();

    let mut current_pacman_package = String::default();
    let mut current_file_package = String::default();

    loop {
        match String::cmp(&current_file_package, &current_pacman_package) {
            Ordering::Equal => {
                current_file_package = match file_packages.next() {
                        None => {
                            for package in pacman_packages {
                                to_install.push(package)
                            }
                            break;
                        },
                        Some(package) => package,
                };
                current_pacman_package = match pacman_packages.next() {
                        None => {
                            for package in file_packages {
                                to_install.push(package)
                            }
                            break;
                        },
                        Some(package) => package,
                };
            },
            Ordering::Less => {
                to_install.push(current_file_package);
                current_file_package = match file_packages.next() {
                        None => {
                            for package in pacman_packages {
                                to_install.push(package)
                            }
                            break;
                        },
                        Some(package) => package,
                };
            },
            Ordering::Greater => {
                to_uninstall.push(current_pacman_package);
                current_pacman_package = match pacman_packages.next() {
                        None => {
                            for package in file_packages {
                                to_install.push(package)
                            }
                            break;
                        },
                        Some(package) => package,
                };
            },
        }
    }

    (to_install, to_uninstall)
}

fn process_package_list<'a>(packages_lines: impl Iterator<Item = &'a str>) -> Result<Vec<String>, Error> {
    let mut packages = packages_lines.map(|str| {
        let mut words = str.split(' ');
        words.next()
    }).collect::<Option<Vec<&str>>>().ok_or_else(|| Error::InvalidPackageList)?;
    packages.sort();
    Ok(packages.into_iter().map(str::to_string).collect())
}

fn get_pacman_packages() -> Result<Vec<String>, Error> {
    let mut pacman = process::Command::new("pacman");
    pacman.arg("-Qe");
    let pacman_bytes = pacman.output().map_err(|_| Error::PacmanNotInstalled)?.stdout;
    let pacman_string = String::from_utf8(pacman_bytes).map_err(|_| Error::InvalidUTF8)?;
    let pacman_lines = pacman_string.lines();
    Ok(process_package_list(pacman_lines)?)
}

fn get_file_packages(config_directory: &Path, recursive: bool) -> Result<Vec<String>, Error> {
    let mut package_list: Vec<String> = Vec::new();

    for entry in fs::read_dir(config_directory).map_err(|_| Error::DirectoryNotFound)? {
        let path = entry.map_err(|_| Error::FileError)?.path();

        if path.is_dir() && recursive {
            package_list.extend(get_file_packages(&path, true)?);
            continue;
        }

        let extension = match path.extension() {
            None => continue,
            Some(x) => x,
        };

        if extension == "packman" {
            let file_contents = fs::read_to_string(path)
                .map_err(|_| Error::FileError)?;
            let file_lines = file_contents.lines()
                .filter(|line| *line != "")
                .filter(|line| !line.starts_with("#"))
                .map(str::trim)
                .map(str::to_string);
            package_list.extend(file_lines.collect::<Vec<String>>())
        }
    }

    Ok(process_package_list(package_list.iter().map(|str| &**str))?)
}

pub enum Error {
    PacmanNotInstalled,
    InvalidUTF8,
    DirectoryNotFound,
    EmptyDirectory,
    FileError,
    InvalidPackageList,
    NoExitCode,
    FailedInstall(Option<i32>),
    FailedUninstall(Option<i32>),
    InvalidArg(String),
    ProvideDirectory,
}

impl Error {
    pub fn print(&self) {
        let message = match &self {
            Error::PacmanNotInstalled => "Pacman is not installed.".to_string(),
            Error::InvalidUTF8 => "The output of pacman -Qe was not valid UTF-8, unable to parse.".to_string(),
            Error::DirectoryNotFound => "The inputted directory was not found. Check that you entered the right location.".to_string(),
            Error::EmptyDirectory => "The inputted directory is empty.".to_string(),
            Error::FileError => "Finding the packages files failed.".to_string(),
            Error::InvalidPackageList => "The formatted package list was unable to be parsed, as one of the lines was empty.".to_string(),
            Error::NoExitCode => "Pacman exited without an error code.".to_string(),
            Error::FailedInstall(exit_code) => match exit_code {
                Some(code) => format!("Pacman failed to install packages with exit code {code}."),
                None => format!("Pacman failed to install packages"),
            },
            Error::FailedUninstall(exit_code) => match exit_code {
                Some(code) => format!("Pacman failed to uninstall packages with exit code {code}."),
                None => format!("Pacman failed to uninstall packages"),
            },
            Error::InvalidArg(arg) => format!("Invalid argument {arg}"),
            Error::ProvideDirectory => "Please provide the configuration directory with the -d or --config-directory flag.".to_string(),
        };
        elogln!("{message}");
    }
}

pub struct Config {
    config_directory: String, // Default ~/.config/packman
    upgrade_system: bool, // Default false
    forbid_uninstall: bool, // Default false
    list_uninstall: bool, // Default false
    recursive: bool, // Default false
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Self, Error> {
        args.next(); // We don't care about the location of this program

        let mut config_directory = format!("{}/.config/packman", env::var("HOME").unwrap());
        let mut upgrade_system = false;
        let mut forbid_uninstall = false;
        let mut list_uninstall = false;
        let mut recursive = false;

        loop {
            let arg = match args.next() {
                None => break,
                Some(x) => x,
            };

            if arg.starts_with("--") {
                if arg == "--config-directory" { config_directory = args.next().ok_or_else(|| Error::ProvideDirectory)? }
                else if arg == "--upgrade" { upgrade_system = true; }
                else if arg == "--forbid-uninstall" { forbid_uninstall = true; }
                else if arg == "--list-uninstall" { list_uninstall = true; }
                else if arg == "--recursive" { recursive = true; }
                else { return Err(Error::InvalidArg(arg)) }
            } else if arg.starts_with("-") {
                for flag in arg.chars() {
                    match flag {
                        '-' => (), // Skip the initial character
                        'd' => config_directory = args.next().ok_or_else(|| Error::ProvideDirectory)?,
                        'u' => upgrade_system = true,
                        'f' => forbid_uninstall = true,
                        'l' => list_uninstall = true,
                        'r' => recursive = true,
                        _ => return Err(Error::InvalidArg(format!("-{flag}"))),
                    }
                }
            }
        }

        Ok(Self {
            config_directory,
            upgrade_system,
            forbid_uninstall,
            list_uninstall,
            recursive,
        })
    }
}

