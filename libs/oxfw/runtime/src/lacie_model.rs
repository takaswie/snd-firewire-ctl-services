// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::{SndUnitExt, FwFcpExt};

use alsactl::CardExtManual;

use core::card_cntr;
use core::elem_value_accessor::ElemValueAccessor;

use ta1394::Ta1394Avc;
use ta1394::audio::{AUDIO_SUBUNIT_0_ADDR, AudioFeature, CtlAttr, FeatureCtl, AudioCh};

use super::common_ctl::CommonCtl;

#[derive(Default, Debug)]
pub struct LacieModel {
    avc: hinawa::FwFcp,
    common_ctl: CommonCtl,
    voluntary: bool,
}

impl<'a> LacieModel {
    const FCP_TIMEOUT_MS: u32 = 100;

    const VOL_LABEL: &'a str = "PCM Playback Volume";
    const MUTE_LABEL: &'a str = "PCM Playback Switch";

    const FB_ID: u8 = 0x01;
}

impl card_cntr::CtlModel<hinawa::SndUnit> for LacieModel {
    fn load(&mut self, unit: &mut hinawa::SndUnit, card_cntr: &mut card_cntr::CardCntr) -> Result<(), Error> {
        self.avc.bind(&unit.get_node())?;

        self.common_ctl.load(&self.avc, card_cntr, Self::FCP_TIMEOUT_MS)?;

        // NOTE: I have a plan to remove control functionality from ALSA oxfw driver for future.
        let elem_id_list = card_cntr.card.get_elem_id_list()?;
        self.voluntary = elem_id_list.iter().find(|elem_id| elem_id.get_name().as_str() == Self::VOL_LABEL).is_none();
        if self.voluntary {
            let mut op = AudioFeature::new(Self::FB_ID, CtlAttr::Minimum, AudioCh::All,
                                           FeatureCtl::Volume(vec![-1]));
            self.avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, Self::FCP_TIMEOUT_MS)?;
            let min = match op.ctl {
                FeatureCtl::Volume(data) => data[0],
                _ => unreachable!(),
            };

            let mut op = AudioFeature::new(Self::FB_ID, CtlAttr::Maximum, AudioCh::All,
                                           FeatureCtl::Volume(vec![-1]));
            self.avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, Self::FCP_TIMEOUT_MS)?;
            let max = match op.ctl {
                FeatureCtl::Volume(data) => data[0],
                _ => unreachable!(),
            };

            let mut op = AudioFeature::new(Self::FB_ID, CtlAttr::Resolution, AudioCh::All,
                                           FeatureCtl::Volume(vec![-1]));
            self.avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, Self::FCP_TIMEOUT_MS)?;
            let step = match op.ctl {
                FeatureCtl::Volume(data) => data[0],
                _ => unreachable!(),
            };

            let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                       0, 0, Self::VOL_LABEL, 0);
            let _ = card_cntr.add_int_elems(&elem_id, 1, min as i32, max as i32, step as i32,
                                            1, None, true)?;

            let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                       0, 0, Self::MUTE_LABEL, 0);
            let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;
        }

        Ok(())
    }

    fn read(&mut self, _: &mut hinawa::SndUnit, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.common_ctl.read(&self.avc, elem_id, elem_value, Self::FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.voluntary {
            match elem_id.get_name().as_str() {
                Self::VOL_LABEL => {
                    ElemValueAccessor::<i32>::set_val(elem_value, || {
                        let mut op = AudioFeature::new(Self::FB_ID, CtlAttr::Current, AudioCh::All,
                                                       FeatureCtl::Volume(vec![-1]));
                        self.avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, Self::FCP_TIMEOUT_MS)?;
                        if let FeatureCtl::Volume(data) = op.ctl {
                            Ok(data[0] as i32)
                        } else {
                            unreachable!();
                        }
                    })?;
                    Ok(true)
                }
                Self::MUTE_LABEL => {
                    ElemValueAccessor::<bool>::set_val(elem_value, || {
                        let mut op = AudioFeature::new(Self::FB_ID, CtlAttr::Current, AudioCh::All,
                                                       FeatureCtl::Mute(vec![false]));
                        self.avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, Self::FCP_TIMEOUT_MS)?;
                        if let FeatureCtl::Mute(data) = op.ctl {
                            Ok(data[0])
                        } else {
                            unreachable!();
                        }
                    })?;
                    Ok(true)
                }
                _ => Ok(false),
            }
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &mut hinawa::SndUnit, elem_id: &alsactl::ElemId, _: &alsactl::ElemValue,
             new: &alsactl::ElemValue) -> Result<bool, Error>
    {
        if self.common_ctl.write(unit, &self.avc, elem_id, new, Self::FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.voluntary {
            match elem_id.get_name().as_str() {
                Self::VOL_LABEL => {
                    ElemValueAccessor::<i32>::get_val(new, |val| {
                        let mut op = AudioFeature::new(Self::FB_ID, CtlAttr::Current, AudioCh::All,
                                                       FeatureCtl::Volume(vec![val as i16]));
                        self.avc.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, Self::FCP_TIMEOUT_MS)
                    })?;
                    Ok(true)
                }
                Self::MUTE_LABEL => {
                    ElemValueAccessor::<bool>::get_val(new, |val| {
                        let mut op = AudioFeature::new(Self::FB_ID, CtlAttr::Current, AudioCh::All,
                                                       FeatureCtl::Mute(vec![val]));
                        self.avc.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, Self::FCP_TIMEOUT_MS)
                    })?;
                    Ok(true)
                }
                _ => Ok(false),
            }
        } else {
            Ok(false)
        }
    }
}

impl card_cntr::NotifyModel<hinawa::SndUnit, bool> for LacieModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<alsactl::ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.notified_elem_list);
    }

    fn parse_notification(&mut self, _: &mut hinawa::SndUnit, _: &bool) -> Result<(), Error> {
        Ok(())
    }

    fn read_notified_elem(&mut self, _: &hinawa::SndUnit, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        self.common_ctl.read(&self.avc, elem_id, elem_value, Self::FCP_TIMEOUT_MS)
    }
}
