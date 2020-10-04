// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use crate::card_cntr;
use card_cntr::CtlModel;

pub struct FirepowerModel;

impl FirepowerModel {
    pub fn new() -> Self {
        FirepowerModel{}
    }
}

impl CtlModel<hinawa::SndUnit> for FirepowerModel {
    fn load(&mut self, _: &hinawa::SndUnit, _: &mut card_cntr::CardCntr) -> Result<(), Error> {
        Ok(())
    }

    fn read(&mut self, _: &hinawa::SndUnit, _: &alsactl::ElemId, _: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        Ok(false)
    }

    fn write(&mut self, _: &hinawa::SndUnit, _: &alsactl::ElemId, _: &alsactl::ElemValue, _: &alsactl::ElemValue)
        -> Result<bool, Error>
    {
        Ok(false)
    }
}
