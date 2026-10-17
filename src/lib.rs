use std::{
    process,
    env,
    fs,
    cmp::Ordering,
};

mod logging;

pub fn run(config: Config) -> Result<(), Error> {
    let config_directory = config.config_directory;

    let pacman_packages = get_pacman_packages()?;
    let file_packages = get_file_packages(config_directory)?;

    let (to_uninstall, to_install) = compare_package_lists(pacman_packages.into_iter(), file_packages.into_iter());
    let _ = install(to_install, config.upgrade_system)?;
    if config.list_uninstall { list_uninstall(to_uninstall.iter(), to_uninstall.len() == 0); } 
    if config.allow_uninstall { let _ = uninstall(to_uninstall)?; }

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

fn install(packages: Vec<String>, upgrade_system: bool) -> Result<(), Error> {
    list_install(packages.iter(), packages.len() == 0);

    let mut pacman = process::Command::new("sudo");
    pacman.arg("pacman");
    if upgrade_system {
        pacman.arg("-Syu");
    } else {
        if packages.len() == 0 { return Ok(()); }
        pacman.arg("-S");
    }

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

fn compare_package_lists<T>(pacman_packages: T, mut file_packages: T) -> (Vec<String>, Vec<String>)
    where T: Iterator<Item = String>
{
    let mut to_uninstall: Vec<String> = Vec::new();
    let mut to_install: Vec<String> = Vec::new();

    for pacman_package in pacman_packages {
        loop {
            let next_file_package = file_packages.next();
            match next_file_package {
                None => {
                    to_uninstall.push(pacman_package.clone());
                    break;
                },
                Some(file_package) => {
                    match file_package.cmp(&pacman_package) {
                        Ordering::Equal => break,
                        Ordering::Less => to_install.push(file_package),
                        Ordering::Greater => {
                            to_uninstall.push(pacman_package);
                            break;
                        },
                    }
                }
            }
        }
    }

    // Install the remaining packages added to the file
    for file_package in file_packages {
        to_install.push(file_package);
    }

    (to_uninstall, to_install)
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

fn get_file_packages(config_directory: String) -> Result<Vec<String>, Error> {
    let mut package_list: Vec<String> = Vec::new();
    for entry in fs::read_dir(config_directory).map_err(|_| Error::DirectoryNotFound)? {
        let file_contents = fs::read_to_string(entry.map_err(|_| Error::FileError)?.path())
            .map_err(|_| Error::FileError)?;
        let file_lines = file_contents.lines()
            .filter(|line| *line != "")
            .filter(|line| !line.starts_with("#"))
            .map(str::trim)
            .map(str::to_string);
        package_list.extend(file_lines.collect::<Vec<String>>())
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
        };
        elogln!("{message}");
    }
}

pub struct Config {
    config_directory: String,
    upgrade_system: bool, // Default false
    allow_uninstall: bool, // Default true
    list_uninstall: bool, // Default false
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Self, Error> {
        args.next(); // We don't care about the location of this program

        let mut upgrade_system = false;
        let mut allow_uninstall = true;
        let mut list_uninstall = false;
        let mut config_directory = format!("{}/.config/packman", env::var("HOME").unwrap());

        for arg in args {
            if arg.starts_with("--") {
                if arg == "--upgrade" { upgrade_system = true; }
                else if arg == "--disallow-uninstall" { allow_uninstall = false; }
                else if arg == "--list-uninstall" { list_uninstall = true; }
                else { return Err(Error::InvalidArg(arg)) }
            } else if arg.starts_with("-") {
                for flag in arg.chars() {
                    match flag {
                        '-' => (), // Skip the initial character
                        'u' => upgrade_system = true,
                        'd' => allow_uninstall = false,
                        'l' => list_uninstall = true,
                        _ => return Err(Error::InvalidArg(flag.to_string())),
                    }
                }
            } else {
                config_directory = arg;
            }
        }

        Ok(Self {
            config_directory,
            upgrade_system,
            allow_uninstall,
            list_uninstall
        })
    }
}

