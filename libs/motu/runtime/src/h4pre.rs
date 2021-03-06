// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::SndMotu;

use core::card_cntr::{CardCntr, CtlModel};

use motu_protocols::version_3::*;

use super::common_ctls::*;
use super::v3_ctls::*;

const TIMEOUT_MS: u32 = 100;

#[derive(Default)]
pub struct H4pre {
    proto: H4preProtocol,
    clk_ctls: V3ClkCtl,
    phone_assign_ctl: CommonPhoneCtl,
}

impl CtlModel<SndMotu> for H4pre {
    fn load(&mut self, _: &mut SndMotu, card_cntr: &mut CardCntr)
        -> Result<(), Error>
    {
        self.clk_ctls.load(&self.proto, card_cntr)?;
        self.phone_assign_ctl.load(&self.proto, card_cntr)?;
        Ok(())
    }

    fn read(&mut self, unit: &mut SndMotu, elem_id: &alsactl::ElemId,
            elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctls.read(unit, &self.proto, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.phone_assign_ctl.read(unit, &self.proto, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &mut SndMotu, elem_id: &alsactl::ElemId, old: &alsactl::ElemValue,
             new: &alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctls.write(unit, &self.proto, elem_id, old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.phone_assign_ctl.write(unit, &self.proto, elem_id, old, new, TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
