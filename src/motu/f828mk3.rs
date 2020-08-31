// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::{SndMotu, FwReq};

use crate::card_cntr::{CardCntr, CtlModel};

use super::v3_clk_ctls::V3ClkCtl;
use super::v3_port_ctls::V3PortCtl;

pub struct F828mk3<'a> {
    req: FwReq,
    clk_ctls: V3ClkCtl<'a>,
    port_ctls: V3PortCtl<'a>,
}

impl<'a> F828mk3<'a> {
    const CLK_RATE_LABELS: &'a [&'a str] = &[
        "44100", "48000",
        "88200", "96000",
        "176400", "192000",
    ];
    const CLK_RATE_VALS: &'a [u8] = &[0x00, 0x01, 0x02, 0x03, 0x04, 0x05];

    const CLK_SRC_LABELS: &'a [&'a str] = &[
        "Internal",
        "Word-clock",
        "S/PDIF-on-coax",
        "Signal-on-opt-A",
        "Signal-on-opt-B",
    ];
    const CLK_SRC_VALS: &'a [u8] = &[0x00, 0x01, 0x10, 0x18, 0x19];

    const PORT_ASSIGN_LABELS: &'a [&'a str] = &[
        "Main-1/2",         // = Stream-11/12
        "Analog-1/2",       // = Stream-3/4
        "Analog-3/4",       // = Stream-5/6
        "Analog-5/6",       // = Stream-7/8
        "Analog-7/8",       // = Stream-9/10
        "S/PDIF-1/2",       // = Stream-13/14
        "Phone-1/2",        // = Stream-1/2
        "Optical-A-1/2",    // = Stream-15/16
        "Optical-A-3/4",    // = Stream-17/18
        "Optical-A-5/6",    // = Stream-19/20
        "Optical-A-7/8",    // = Stream-21/22
        "Optical-B-1/2",    // = Stream-23/24
        "Optical-B-3/4",    // = Stream-25/26
        "Optical-B-5/6",    // = Stream-27/28
        "Optical-B-7/8",    // = Stream-29/30
    ];
    const PORT_ASSIGN_VALS: &'a [u8] = &[
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
        0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
    ];

    pub fn new() -> Self {
        F828mk3{
            req: FwReq::new(),
            clk_ctls: V3ClkCtl::new(Self::CLK_RATE_LABELS, Self::CLK_RATE_VALS,
                                    Self::CLK_SRC_LABELS, Self::CLK_SRC_VALS, true),
            port_ctls: V3PortCtl::new(Self::PORT_ASSIGN_LABELS, Self::PORT_ASSIGN_VALS,
                                      true, true, true, true),
        }
    }
}

impl<'a> CtlModel<SndMotu> for F828mk3<'a> {
    fn load(&mut self, unit: &SndMotu, card_cntr: &mut CardCntr)
        -> Result<(), Error>
    {
        self.clk_ctls.load(unit, card_cntr)?;
        self.port_ctls.load(unit, card_cntr)?;
        Ok(())
    }

    fn read(&mut self, unit: &SndMotu, elem_id: &alsactl::ElemId,
            elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctls.read(unit, &self.req, elem_id, elem_value)? {
            Ok(true)
        } else if self.port_ctls.read(unit, &self.req, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &SndMotu, elem_id: &alsactl::ElemId, old: &alsactl::ElemValue,
             new: &alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctls.write(unit, &self.req, elem_id, old, new)? {
            Ok(true)
        } else if self.port_ctls.write(unit, &self.req, elem_id, old, new)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}