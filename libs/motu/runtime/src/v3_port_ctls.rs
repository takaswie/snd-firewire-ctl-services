// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use hinawa::{SndUnitExt, SndMotu};

use core::card_cntr::CardCntr;
use core::elem_value_accessor::ElemValueAccessor;

use super::common_proto::CommonProto;
use super::v3_proto::V3Proto;

pub struct V3PortCtl<'a> {
    assign_labels: &'a [&'a str],
    assign_vals: &'a [u8],
    has_main_assign: bool,
    has_return_assign: bool,
    has_word_bnc: bool,
    has_opt_ifaces: bool,

    pub notified_elems: Vec<alsactl::ElemId>,
}

impl<'a> V3PortCtl<'a> {
    const PHONE_ASSIGN_NAME: &'a str = "phone-assign";
    const MAIN_ASSIGN_NAME: &'a str = "main-assign";
    const RETURN_ASSIGN_NAME: &'a str = "return-assign";
    const WORD_OUT_MODE_NAME: &'a str = "word-out-mode";
    const OPT_IFACE_IN_MODE_NAME: &'a str = "optical-iface-in-mode";
    const OPT_IFACE_OUT_MODE_NAME: &'a str = "optical-iface-out-mode";

    const WORD_OUT_MODE_LABELS: &'a [&'a str] = &[
        "Force 44.1/48.0 kHz",
        "Follow to system clock",
    ];
    const WORD_OUT_MODE_VALS: &'a [u8] = &[0x00, 0x01];

    const OPT_IFACE_MODE_LABELS: &'a [&'a str] = &[
        "None",
        "ADAT",
        "S/PDIF",
    ];

    pub fn new(assign_labels: &'a [&'a str], assign_vals: &'a [u8], has_main_assign: bool,
               has_return_assign: bool, has_opt_ifaces: bool, has_word_bnc: bool) -> Self {
        V3PortCtl{
            assign_labels,
            assign_vals,
            has_main_assign,
            has_return_assign,
            has_word_bnc,
            has_opt_ifaces,
            notified_elems: Vec::new(),
        }
    }

    pub fn load(&mut self, _: &SndMotu, card_cntr: &mut CardCntr)
        -> Result<(), Error>
    {
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                   0, 0, Self::PHONE_ASSIGN_NAME, 0);
        let elem_id_list = card_cntr.add_enum_elems(&elem_id, 1, 1, &self.assign_labels, None, true)?;
        self.notified_elems.extend_from_slice(&elem_id_list);

        if self.has_main_assign {
            let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                       0, 0, Self::MAIN_ASSIGN_NAME, 0);
            let elem_id_list = card_cntr.add_enum_elems(&elem_id, 1, 1, &self.assign_labels, None, true)?;
            self.notified_elems.extend_from_slice(&elem_id_list);
        }

        if self.has_return_assign {
            let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                       0, 0, Self::RETURN_ASSIGN_NAME, 0);
            let elem_id_list = card_cntr.add_enum_elems(&elem_id, 1, 1, &self.assign_labels, None, true)?;
            self.notified_elems.extend_from_slice(&elem_id_list);
        }

        if self.has_word_bnc {
            let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card,
                                                       0, 0, Self::WORD_OUT_MODE_NAME, 0);
            let elem_id_list = card_cntr.add_enum_elems(&elem_id, 1, 1,
                                                        Self::WORD_OUT_MODE_LABELS, None, true)?;
            self.notified_elems.extend_from_slice(&elem_id_list);
        }

        if self.has_opt_ifaces {
            let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                       0, 0, Self::OPT_IFACE_IN_MODE_NAME, 0);
            let _ = card_cntr.add_enum_elems(&elem_id, 1, 2, Self::OPT_IFACE_MODE_LABELS, None, true)?;

            let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                       0, 0, Self::OPT_IFACE_OUT_MODE_NAME, 0);
            let _ = card_cntr.add_enum_elems(&elem_id, 1, 2, Self::OPT_IFACE_MODE_LABELS, None, true)?;
        }

        Ok(())
    }

    fn get_opt_iface_mode(&mut self, unit: &SndMotu, req: &hinawa::FwReq, is_out: bool, is_b: bool)
        -> Result<u32, Error>
    {
        let (enabled, no_adat) = req.get_opt_iface_mode(unit, is_out, is_b)?;

        let idx = match enabled {
            false => 0,
            true => {
                match no_adat {
                    false => 1,
                    true => 2,
                }
            }
        };
        Ok(idx)
    }

    fn set_opt_iface_mode(&mut self, unit: &SndMotu, req: &hinawa::FwReq, is_out: bool, is_b: bool,
                          mode: u32)
        -> Result<(), Error>
    {
        let (enabled, no_adat) = match mode {
            0 => (false, false),
            1 => (true, false),
            2 => (true, true),
            _ => {
                let label = format!("Invalid argument for optical interface: {}", mode);
                return Err(Error::new(FileError::Nxio, &label));
            }
        };
        req.set_opt_iface_mode(unit, is_out, is_b, enabled, no_adat)
    }

    pub fn read(&mut self, unit: &SndMotu, req: &hinawa::FwReq, elem_id: &alsactl::ElemId,
            elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::PHONE_ASSIGN_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let val = req.get_phone_assign(unit, &self.assign_vals)?;
                    Ok(val as u32)
                })?;
                Ok(true)
            }
            Self::MAIN_ASSIGN_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let val = req.get_main_assign(unit, &self.assign_vals)?;
                    Ok(val as u32)
                })?;
                Ok(true)
            }
            Self::RETURN_ASSIGN_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let val = req.get_return_assign(unit, &self.assign_vals)?;
                    Ok(val as u32)
                })?;
                Ok(true)
            }
            Self::WORD_OUT_MODE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let val = req.get_word_out(unit, &Self::WORD_OUT_MODE_VALS)?;
                    Ok(val as u32)
                })?;
                Ok(true)
            }
            Self::OPT_IFACE_IN_MODE_NAME => {
                ElemValueAccessor::<u32>::set_vals(elem_value, 2, |idx| {
                    let val = self.get_opt_iface_mode(unit, req, false, idx > 0)?;
                    Ok(val)
                })?;
                Ok(true)
            }
            Self::OPT_IFACE_OUT_MODE_NAME => {
                ElemValueAccessor::<u32>::set_vals(elem_value, 2, |idx| {
                    let val = self.get_opt_iface_mode(unit, req, true, idx > 0)?;
                    Ok(val)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(&mut self, unit: &SndMotu, req: &hinawa::FwReq, elem_id: &alsactl::ElemId,
                 old: &alsactl::ElemValue, new: &alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::PHONE_ASSIGN_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    req.set_phone_assign(unit, &self.assign_vals, val as usize)
                })?;
                Ok(true)
            }
            Self::MAIN_ASSIGN_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    req.set_main_assign(unit, &self.assign_vals, val as usize)
                })?;
                Ok(true)
            }
            Self::RETURN_ASSIGN_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    req.set_return_assign(unit, &self.assign_vals, val as usize)
                })?;
                Ok(true)
            }
            Self::WORD_OUT_MODE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    req.set_word_out(unit, &Self::WORD_OUT_MODE_VALS, val as usize)
                })?;
                Ok(true)
            }
            Self::OPT_IFACE_IN_MODE_NAME => {
                unit.lock()?;
                let res = ElemValueAccessor::<u32>::get_vals(new, old, 2, |idx, val| {
                    self.set_opt_iface_mode(unit, req, false, idx > 0, val)
                });
                let _ = unit.unlock();
                res.and(Ok(true))
            }
            Self::OPT_IFACE_OUT_MODE_NAME => {
                unit.lock()?;
                let res = ElemValueAccessor::<u32>::get_vals(new, old, 2, |idx, val| {
                    self.set_opt_iface_mode(unit, req, true, idx > 0, val)
                });
                let _ = unit.unlock();
                res.and(Ok(true))
            }
            _ => Ok(false),
        }
    }
}