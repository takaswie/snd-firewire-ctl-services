// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use hinawa::{FwNodeExtManual, SndUnitExt, SndTscmExt};

use crate::ieee1212;

use super::isoc_console_unit::IsocConsoleUnit;

pub enum TascamUnit {
    IsocConsole(IsocConsoleUnit),
    Asynch,
}

impl TascamUnit {
    pub fn new(subsystem: &String, sysnum: u32) -> Result<Self, Error> {
        let unit = match subsystem.as_str() {
            "snd" => {
                let unit = hinawa::SndTscm::new();
                let devnode = format!("/dev/snd/hwC{}D0", sysnum);
                unit.open(&devnode)?;

                let node = unit.get_node();
                let name = detect_model_name(&node)?;
                let isoc_unit = match name.as_str() {
                    "FW-1884" | "FW-1082" => IsocConsoleUnit::new(unit, name, sysnum),
                    _ => Err(Error::new(FileError::Noent, "Not supported")),
                }?;

                Self::IsocConsole(isoc_unit)
            }
            "fw" => Self::Asynch,
            _ => {
                let label = "Invalid name of subsystem";
                return Err(Error::new(FileError::Nodev, &label));
            }
        };

        Ok(unit)
    }

    pub fn listen(&mut self) -> Result<(), Error> {
        match self {
            Self::IsocConsole(unit) => unit.listen(),
            Self::Asynch => Ok(()),
        }
    }

    pub fn run(&mut self) {
        match self {
            Self::IsocConsole(unit) => unit.run(),
            Self::Asynch => (),
        }
    }
}

fn detect_model_name(node: &hinawa::FwNode) -> Result<String, Error> {
    let data = node.get_config_rom()?;

    ieee1212::get_root_entry_list(data).iter().find_map(|entry| {
        if entry.key == ieee1212::KeyType::Unit as u8 {
            if let ieee1212::EntryData::Directory(dir) = &entry.data {
                dir.iter().find_map(|de| {
                    if de.key == ieee1212::KeyType::DependentInfo as u8 {
                        if let ieee1212::EntryData::Directory(d) = &de.data {
                            d.iter().find_map(|e| {
                                if e.key == ieee1212::KeyType::BusDependentInfo as u8 {
                                    if let ieee1212::EntryData::Leaf(l) = &e.data {
                                        ieee1212::parse_leaf_entry_as_text(l)
                                    } else {
                                        None
                                    }
                                } else{
                                    None
                                }
                            })
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
            } else {
                None
            }
        } else {
            None
        }
    }).ok_or_else(|| {
        let label = "Invalid format of configuration ROM";
        Error::new(FileError::Nxio, &label)
    })
}
