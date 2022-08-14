use colored::Colorize;
use std::any::Any;
use std::cmp::PartialEq;
use std::fmt;
use std::fs::{self, read_dir};
use std::path::Path;

use super::create_issue;
use super::Error;

const LID_STATUS_PATH: [&str; 4] = [
    "/proc/acpi/button/lid/LID/state",
    "/proc/acpi/button/lid/LID0/state",
    "/proc/acpi/button/lid/LID1/state",
    "/proc/acpi/button/lid/LID2/state",
];

const BATTERY_CHARGE_PATH: [&str; 4] = [
    "/sys/class/power_supply/BAT/capacity",
    "/sys/class/power_supply/BAT0/capacity",
    "/sys/class/power_supply/BAT1/capacity",
    "/sys/class/power_supply/BAT2/capacity",
];

const POWER_SOURCE_PATH: [&str; 4] = [
    "/sys/class/power_supply/AC/online",
    "/sys/class/power_supply/AC0/online",
    "/sys/class/power_supply/AC1/online",
    "/sys/class/power_supply/ACAD/online",
];

#[derive(PartialEq, Eq)]
pub enum LidState {
    Open,
    Closed,
    Unapplicable,
    Unknown,
}

impl fmt::Display for LidState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LidState::Open => write!(f, "open"),
            LidState::Closed => write!(f, "closed"),
            LidState::Unapplicable => write!(f, "unapplicable"),
            LidState::Unknown => write!(f, "unknown"),
        }
    }
}

pub fn has_battery() -> bool {
    let power_dir = Path::new("/sys/class/power_supply/");
    let dir_count = read_dir(power_dir).into_iter().len();
    dir_count > 0
}

pub fn get_best_path(paths: [&'static str; 4]) -> Result<&str, Error> {
    for path in paths.iter() {
        if Path::new(path).exists() {
            return Ok(path);
        }
    }

    Err(Error::Unknown)
}

pub fn read_lid_state() -> Result<LidState, Error> {
    let path: &str = match get_best_path(LID_STATUS_PATH) {
        Ok(path) => path,
        Err(error) => {
            if error.type_id() == Error::IO.type_id() {
                // Make sure to return IO error if one occurs
                return Err(error);
            }
            eprintln!("Could not detect your lid state.");
            create_issue!("If you are on a laptop");
            return Ok(LidState::Unapplicable);
        }
    };

    let lid_str = fs::read_to_string(path)?;

    let state = if lid_str.contains("open") {
        LidState::Open
    } else if lid_str.contains("closed") {
        LidState::Closed
    } else {
        LidState::Unknown
    };

    Ok(state)
}

pub fn get_battery_status(charging: bool) -> String {
    if has_battery() {
        match read_battery_charge() {
            Ok(bat) => {
                format!(
                    "Battery: {}",
                    if charging {
                        format!("{}%", bat).green()
                    } else {
                        format!("{}%", bat).red()
                    },
                )
            }
            Err(e) => format!("Battery charge could not be read\n{:?}", e),
        }
    } else {
        format!("Battery: {}", "N/A".bold())
    }
}

pub fn read_battery_charge() -> Result<i8, Error> {
    let path: &str = match get_best_path(BATTERY_CHARGE_PATH) {
        Ok(path) => path,
        Err(error) => {
            if error.type_id() == Error::IO.type_id() {
                // Make sure to return IO error if one occurs
                return Err(error);
            }
            // If it doesn't exist then it is plugged in so make it 100% percent capacity
            eprintln!("We could not detect your battery.");
            create_issue!("If you are on a laptop");
            return Ok(100);
        }
    };

    let mut cap_str = fs::read_to_string(path)?;

    // Remove the \n char
    cap_str.pop();

    Ok(cap_str.parse::<i8>().unwrap())
}

pub fn read_power_source() -> Result<bool, Error> {
    let path: &str = match get_best_path(POWER_SOURCE_PATH) {
        Ok(path) => path,
        Err(error) => {
            if error.type_id() == Error::IO.type_id() {
                // Make sure to return IO error if one occurs
                return Err(error);
            }
            eprintln!("We could not detect your AC power source.");
            create_issue!("If you have a power source");
            return Ok(true);
        }
    };

    let mut pwr_str = fs::read_to_string(path)?;

    // Remove the \n char
    pwr_str.pop();

    Ok(pwr_str == "1")
}
