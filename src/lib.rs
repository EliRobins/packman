use std::{
    process,
    env,
    fs,
    cmp::Ordering,
};

mod logging;

pub fn run(config: Config) -> Result<(), Error> {
    let file_location = config.file_location;

    let pacman_packages = get_pacman_packages()?;
    let file_packages = get_file_packages(file_location)?;

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
        log!(" {package}");
    }
    logln!(".");
}

fn list_uninstall<'a>(packages: impl Iterator<Item = &'a String>, empty: bool) {
    if empty {
        logln!("No packages to uninstall.");
        return;
    }

    log!("Uninstalling");
    for package in packages {
        log!(" {package}");
    }
    logln!(".");
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

fn process_package_list<'a>(packages_lines: impl Iterator<Item = &'a str>) -> Vec<String> {
    let mut packages = packages_lines.map(|str| {
        let mut words = str.split(' ');
        words.next().expect("each line should have a valid output")
    }).collect::<Vec<&str>>();
    packages.sort();
    packages.into_iter().map(str::to_string).collect()
}

fn get_pacman_packages() -> Result<Vec<String>, Error> {
    let mut pacman = process::Command::new("pacman");
    pacman.arg("-Qe");
    let pacman_bytes = pacman.output().map_err(|_| Error::PacmanNotInstalled)?.stdout;
    let pacman_string = String::from_utf8(pacman_bytes).map_err(|_| Error::InvalidUTF8)?;
    let pacman_lines = pacman_string.lines();
    Ok(process_package_list(pacman_lines))
}

fn get_file_packages(file_location: String) -> Result<Vec<String>, Error> {
    let file_contents = fs::read_to_string(file_location).map_err(|_| Error::FileNotFound)?;
    let file_lines = file_contents.lines()
        .filter(|line| *line != "")
        .filter(|line| !line.starts_with("#"))
        .filter(|line| !line.starts_with("//"))
        .map(str::trim);
    Ok(process_package_list(file_lines))
}

pub enum Error {
    PacmanNotInstalled,
    InvalidUTF8,
    FileNotFound,
    NotEnoughArgs,
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
            Error::FileNotFound => "The inputted file was not found. Check that you entered the right location.".to_string(),
            Error::NotEnoughArgs => "Please enter a file to update packages with.".to_string(),
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
        eprintln!("PACKMAN: {message}");
    }
}

pub struct Config {
    file_location: String,
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
        let mut file_location = format!("{}/dotfiles/packman/packman.conf", env::var("HOME").unwrap());

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
                file_location = arg;
            }
        }

        Ok(Self {
            file_location,
            upgrade_system,
            allow_uninstall,
            list_uninstall
        })
    }
}

