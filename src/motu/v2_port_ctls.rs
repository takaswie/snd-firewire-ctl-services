// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::{SndUnitExt, SndMotu};
use alsactl::{ElemValueExt, ElemValueExtManual};

use crate::card_cntr::CardCntr;

use super::common_proto::CommonProto;
use super::v2_proto::V2Proto;

pub struct V2PortCtl<'a> {
    phone_assign_labels: &'a [&'a str],
    phone_assign_vals: &'a [u8],
    has_main_vol: bool,
    has_word_bnc: bool,
    has_opt_ifaces: bool,
    has_spdif_opt: bool,

    pub notified_elems: Vec<alsactl::ElemId>,
}

impl<'a> V2PortCtl<'a> {
    const PHONE_ASSIGN_NAME: &'a str = "phone-assign";
    const MAIN_VOL_TARGET_NAME: &'a str = "main-volume-target";
    const WORD_OUT_MODE_NAME: &'a str = "word-out-mode";
    const OPT_IN_IFACE_MODE_NAME: &'a str = "optical-iface-in-mode";
    const OPT_OUT_IFACE_MODE_NAME: &'a str = "optical-iface-out-mode";

    const MAIN_VOL_TARGET_LABELS: &'a [&'a str] = &[
        "Main-out-1/2",
        "Analog-1/2/3/4/5/6",
        "Analog-1/2/3/4/5/6/7/8",
        "S/PDIF-1/2",
    ];
    const MAIN_VOL_TARGET_VALS: &'a [u8] = &[0x00, 0x01, 0x02, 0x03];

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
    const OPT_IFACE_MODE_VALS: &'a [u8] = &[0x00, 0x01, 0x02];

    pub fn new(phone_assign_labels: &'a [&str], phone_assign_vals: &'a [u8], has_main_vol: bool,
               has_word_bnc: bool, has_opt_ifaces: bool, has_spdif_opt: bool) -> Self {
        V2PortCtl{
            phone_assign_labels,
            phone_assign_vals,
            has_main_vol,
            has_word_bnc,
            has_opt_ifaces,
            has_spdif_opt,
            notified_elems: Vec::new(),
        }
    }

    pub fn load(&mut self, _: &SndMotu, card_cntr: &mut CardCntr)
        -> Result<(), Error>
    {
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                   0, 0, Self::PHONE_ASSIGN_NAME, 0);
        let elem_id_list = card_cntr.add_enum_elems(&elem_id, 1, 1, &self.phone_assign_labels, None, true)?;
        self.notified_elems.extend_from_slice(&elem_id_list);

        if self.has_main_vol {
            let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card,
                                                       0, 0, Self::MAIN_VOL_TARGET_NAME, 0);
            let elem_id_list = card_cntr.add_enum_elems(&elem_id, 1, 1, &Self::MAIN_VOL_TARGET_LABELS,
                                                        None, true)?;
            self.notified_elems.extend_from_slice(&elem_id_list);
        }

        if self.has_word_bnc {
            let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card,
                                                       0, 0, Self::WORD_OUT_MODE_NAME, 0);
            let elem_id_list = card_cntr.add_enum_elems(&elem_id, 1, 1, Self::WORD_OUT_MODE_LABELS,
                                                        None, true)?;
            self.notified_elems.extend_from_slice(&elem_id_list);
        }

        if self.has_opt_ifaces {
            let mut labels: Vec<&str> = Self::OPT_IFACE_MODE_LABELS.iter().map(|&l| l).collect();
            if !self.has_spdif_opt {
                labels.pop();
            }

            let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                       0, 0, Self::OPT_IN_IFACE_MODE_NAME, 0);
            let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

            let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                       0, 0, Self::OPT_OUT_IFACE_MODE_NAME, 0);
            let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;
        }

        Ok(())
    }

    pub fn read(&mut self, unit: &SndMotu, req: &hinawa::FwReq, elem_id: &alsactl::ElemId,
            elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::PHONE_ASSIGN_NAME => {
                let val = req.get_phone_assign(unit, &self.phone_assign_vals)?;
                elem_value.set_enum(&[val as u32]);
                Ok(true)
            }
            Self::MAIN_VOL_TARGET_NAME => {
                let val = req.get_main_vol_assign(unit, &Self::MAIN_VOL_TARGET_VALS)?;
                elem_value.set_enum(&[val as u32]);
                Ok(true)
            }
            Self::WORD_OUT_MODE_NAME => {
                let val = req.get_word_out(unit, &Self::WORD_OUT_MODE_VALS)?;
                elem_value.set_enum(&[val as u32]);
                Ok(true)
            }
            Self::OPT_IN_IFACE_MODE_NAME => {
                let val = req.get_opt_in_iface_mode(unit, &Self::OPT_IFACE_MODE_VALS)?;
                elem_value.set_enum(&[val as u32]);
                Ok(true)
            }
            Self::OPT_OUT_IFACE_MODE_NAME => {
                let val = req.get_opt_out_iface_mode(unit, &Self::OPT_IFACE_MODE_VALS)?;
                elem_value.set_enum(&[val as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(&mut self, unit: &SndMotu, req: &hinawa::FwReq, elem_id: &alsactl::ElemId,
                 _: &alsactl::ElemValue, new: &alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::PHONE_ASSIGN_NAME => {
                let mut vals = [0];
                new.get_enum(&mut vals);
                req.set_phone_assign(unit, &self.phone_assign_vals, vals[0] as usize)?;
                Ok(true)
            }
            Self::MAIN_VOL_TARGET_NAME => {
                let mut vals = [0];
                new.get_enum(&mut vals);
                req.set_main_vol_assign(unit, &Self::MAIN_VOL_TARGET_VALS, vals[0] as usize)?;
                Ok(true)
            }
            Self::WORD_OUT_MODE_NAME => {
                let mut vals = [0];
                new.get_enum(&mut vals);
                req.set_word_out(unit, &Self::WORD_OUT_MODE_VALS, vals[0] as usize)?;
                Ok(true)
            }
            Self::OPT_IN_IFACE_MODE_NAME => {
                let mut vals = [0];
                new.get_enum(&mut vals);
                unit.lock()?;
                let res = req.set_opt_in_iface_mode(unit, &Self::OPT_IFACE_MODE_VALS, vals[0] as usize);
                unit.unlock()?;
                match res {
                    Ok(()) => Ok(true),
                    Err(err) => Err(err),
                }
            }
            Self::OPT_OUT_IFACE_MODE_NAME => {
                let mut vals = [0];
                new.get_enum(&mut vals);
                unit.lock()?;
                let res = req.set_opt_out_iface_mode(unit, &Self::OPT_IFACE_MODE_VALS, vals[0] as usize);
                unit.unlock()?;
                match res {
                    Ok(()) => Ok(true),
                    Err(err) => Err(err),
                }
            }
            _ => Ok(false),
        }
    }
}