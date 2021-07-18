// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use glib::Error;

use hinawa::FwFcpExt;
use hinawa::{SndUnit, SndUnitExt};

use alsactl::{ElemId, ElemIfaceType, ElemValue};

use alsa_ctl_tlv_codec::items::{DbInterval, CTL_VALUE_MUTE};

use core::card_cntr::*;
use core::elem_value_accessor::ElemValueAccessor;

use ta1394::Ta1394Avc;
use ta1394::audio::{AUDIO_SUBUNIT_0_ADDR, AudioFeature, FeatureCtl, CtlAttr, AudioCh};

use bebob_protocols::{*, stanton::*};

use super::common_ctls::*;
use super::model::OUT_VOL_NAME;

const FCP_TIMEOUT_MS: u32 = 100;

#[derive(Default)]
pub struct ScratchampModel {
    avc: BebobAvc,
    clk_ctl: ClkCtl,
}

#[derive(Default)]
struct ClkCtl(Vec<ElemId>);

impl MediaClkFreqCtlOperation<ScratchampClkProtocol> for ClkCtl {}

impl SamplingClkSrcCtlOperation<ScratchampClkProtocol> for ClkCtl {
    const SRC_LABELS: &'static [&'static str] = &[
        "Internal",
    ];
}

impl CtlModel<SndUnit> for ScratchampModel {
    fn load(&mut self, unit: &mut SndUnit, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.avc.as_ref().bind(&unit.get_node())?;

        self.clk_ctl.load_freq(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.clk_ctl.load_src(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        InputCtl::load(&self.avc, card_cntr)?;

        Ok(())
    }

    fn read(&mut self, _: &mut SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctl.read_freq(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.clk_ctl.read_src(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if InputCtl::read(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &mut SndUnit, elem_id: &ElemId, old: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctl.write_freq(unit, &self.avc, elem_id, old, new, FCP_TIMEOUT_MS * 3)? {
            Ok(true)
        } else if self.clk_ctl.write_src(unit, &self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if InputCtl::write(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<SndUnit, bool> for ScratchampModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.clk_ctl.0);
    }

    fn parse_notification(&mut self, _: &mut SndUnit, _: &bool) -> Result<(), Error> {
        Ok(())
    }

    fn read_notified_elem(&mut self, _: &SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        self.clk_ctl.read_freq(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)
    }
}

const VOL_MIN: i32 = i16::MIN as i32;
const VOL_MAX: i32 = 0x0000;
const VOL_STEP: i32 = 0x0080;
const VOL_TLV: DbInterval = DbInterval{min: -12800, max: 0, linear: false, mute_avail: true};

const OUTPUT_LABELS: &[&str] = &[
    "analog-1", "analog-2", "analog-3", "analog-4",
    "headphone-1", "headphone-2",
];

const FB_IDS: [u8;3] = [1, 2, 3];

trait InputCtl : Ta1394Avc {
    fn load(&self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        // For volume of outputs.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUT_VOL_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, VOL_MIN, VOL_MAX, VOL_STEP,
                                        OUTPUT_LABELS.len(),
                                        Some(&Into::<Vec<u32>>::into(VOL_TLV)), true)?;

        Ok(())
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            OUT_VOL_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, OUTPUT_LABELS.len(), |idx| {
                    let func_blk_id = FB_IDS[idx / 2];
                    let audio_ch_num = AudioCh::Each((idx % 2) as u8);
                    let mut op = AudioFeature::new(func_blk_id, CtlAttr::Current, audio_ch_num,
                                                   FeatureCtl::Volume(vec![-1]));
                    self.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)?;
                    if let FeatureCtl::Volume(data) = op.ctl {
                        let val = if data[0] == FeatureCtl::NEG_INFINITY { CTL_VALUE_MUTE } else { data[0] as i32 };
                        Ok(val)
                    } else {
                        unreachable!();
                    }
                })?;
                Ok(true)
            },
            _ => Ok(false),
        }
    }

    fn write(&self, elem_id: &ElemId, old: &ElemValue, new: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            OUT_VOL_NAME => {
                ElemValueAccessor::<i32>::get_vals(new, old, OUTPUT_LABELS.len(), |idx, val| {
                    let func_blk_id = FB_IDS[idx / 2];
                    let audio_ch_num = AudioCh::Each((idx % 2) as u8);
                    let v = if val == CTL_VALUE_MUTE { FeatureCtl::NEG_INFINITY } else { val as i16 };
                    let mut op = AudioFeature::new(func_blk_id, CtlAttr::Current, audio_ch_num,
                                                   FeatureCtl::Volume(vec![v]));
                    self.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)
                })?;
                Ok(true)
            },
            _ => Ok(false),
        }
    }
}

impl InputCtl for BebobAvc {}

#[cfg(test)]
mod test {
    use super::*;
    use alsactl::CardError;

    #[test]
    fn test_clk_ctl_definition() {
        let mut card_cntr = CardCntr::new();
        let mut ctl = ClkCtl::default();

        let error = ctl.load_freq(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));
    }
}
