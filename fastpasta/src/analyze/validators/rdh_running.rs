//! Performs running (stateful) checks on [RDH]s.

use std::fmt::Write;

use crate::input::prelude::{Rdh1, Rdh2, RDH};

/// Performs running (stateful) checks on [RDH]s.
pub struct RdhCruRunningChecker<T: RDH> {
    expect_pages_counter: u16,
    // The first 2 RDHs are used to determine what the expected page counter increments are
    first_rdh_cru: Option<T>,
    second_rdh_cru: Option<T>,
    expect_pages_counter_increment: u16,
    last_rdh_cru: Option<T>,
}

impl<T: RDH> Default for RdhCruRunningChecker<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: RDH> RdhCruRunningChecker<T> {
    /// Creates a new [RdhCruRunningChecker]
    pub fn new() -> Self {
        Self {
            expect_pages_counter: 0,
            first_rdh_cru: None,
            second_rdh_cru: None,
            expect_pages_counter_increment: 1,
            last_rdh_cru: None,
        }
    }

    /// Does running checks across CDPs maintaining state based on the previous RDH
    ///
    /// No checks that are dependent on CDP payload state are done here (instead see cdp_running.rs)
    #[inline]
    pub fn check(&mut self, rdh: &T) -> Result<(), String> {
        if self.first_rdh_cru.is_none() {
            self.first_rdh_cru = Some(T::load(&mut rdh.to_byte_slice()).unwrap());
        } else if self.second_rdh_cru.is_none() {
            self.second_rdh_cru = Some(T::load(&mut rdh.to_byte_slice()).unwrap());
            self.expect_pages_counter_increment =
                self.second_rdh_cru.as_ref().unwrap().rdh2().pages_counter;
        }

        let mut err_str = String::new();
        let mut err_cnt: u8 = 0;

        if let Err(e) = self.check_stop_bit_and_page_counter(rdh.rdh2()) {
            err_cnt += 1;
            err_str.push_str(&e);
        };

        if let Err(e) = self.check_orbit_counter_changes(rdh.rdh1()) {
            err_cnt += 1;
            err_str.push_str(&e);
        };

        if let Err(e) = self.check_orbit_trigger_det_field_feeid_same_when_page_not_0(rdh) {
            err_cnt += 1;
            err_str.push_str(&e);
        }

        self.last_rdh_cru = Some(T::load(&mut rdh.to_byte_slice()).unwrap());

        if err_cnt != 0 {
            err_str.insert_str(0, "[E11] RDH running check failed: ");
            return Err(err_str);
        }

        Ok(())
    }

    /// # Check `stop_bit` and `pages_counter` across a CDP
    ///
    /// 1. If `stop_bit` is 0, page counter should be equal to either:
    ///      * the previous `pages_counter` + 1
    ///      * 0 if the `stop_bit` of the previous `Rdh2` was 1 (handled in the match on `stop_bit == 1`)
    ///
    ///     Side effect: `self.expect_pages_counter += 1`
    ///
    /// 2. If `stop_bit` is 1, `pages_counter` should be equal to the previous `pages_counter` + 1
    ///
    ///    Side effect: `self.expect_pages_counter = 0`
    #[inline]
    fn check_stop_bit_and_page_counter(&mut self, rdh2: &Rdh2) -> Result<(), String> {
        let mut err_str = String::new();
        let mut err_cnt: u8 = 0;
        match rdh2.stop_bit {
            0 => {
                if rdh2.pages_counter != self.expect_pages_counter {
                    err_cnt += 1;
                    let tmp = rdh2.pages_counter;
                    write!(
                        err_str,
                        "pages_counter = {tmp} expected: {}.",
                        self.expect_pages_counter
                    )
                    .unwrap();
                }
                self.expect_pages_counter += self.expect_pages_counter_increment;
            }
            1 => {
                if rdh2.pages_counter != self.expect_pages_counter {
                    err_cnt += 1;
                    let tmp = rdh2.pages_counter;
                    write!(
                        err_str,
                        "pages_counter = {tmp} expected: {}.",
                        self.expect_pages_counter
                    )
                    .unwrap();
                }
                self.expect_pages_counter = 0;
            }
            _ => {
                err_cnt += 1;
                let tmp = rdh2.stop_bit;
                write!(err_str, "stop_bit = {tmp}.").unwrap();
            }
        };

        if err_cnt != 0 {
            return Err(err_str);
        }

        Ok(())
    }

    /// If the previous stop bit was 1, the current RDH's orbit counter should be different
    #[inline]
    fn check_orbit_counter_changes(&self, rdh1: &Rdh1) -> Result<(), String> {
        if self.last_rdh_cru.as_ref().is_some_and(|last_rdh_cru| {
            last_rdh_cru.stop_bit() == 1 && last_rdh_cru.rdh1().orbit == rdh1.orbit
        }) {
            let current_orbit = rdh1.orbit;
            return Err(format!("Orbit same as previous {current_orbit}."));
        }
        Ok(())
    }

    /// IF the page counter is not 0, the orbit, trigger, detector and feeid should be the same as the previous RDH
    #[inline]
    fn check_orbit_trigger_det_field_feeid_same_when_page_not_0(
        &self,
        rdh_cru: &T,
    ) -> Result<(), String> {
        let mut err_str = String::new();
        let mut err_cnt: u8 = 0;

        if rdh_cru.pages_counter() != 0 {
            if let Some(last_rdh_cru) = &self.last_rdh_cru {
                if rdh_cru.rdh1().orbit != last_rdh_cru.rdh1().orbit {
                    err_cnt += 1;
                    let tmp_current_orbit = rdh_cru.rdh1().orbit;
                    let tmp_last_orbit = last_rdh_cru.rdh1().orbit;
                    write!(
                        err_str,
                        "Orbit changed from {tmp_last_orbit:#X} to {tmp_current_orbit:#X}."
                    )
                    .unwrap()
                }
                if rdh_cru.rdh2().trigger_type != last_rdh_cru.rdh2().trigger_type {
                    err_cnt += 1;
                    let tmp_current_trigger_type = rdh_cru.rdh2().trigger_type;
                    let tmp_last_trigger_type = last_rdh_cru.rdh2().trigger_type;
                    write!(
                        err_str,
                        "Trigger type changed from {tmp_last_trigger_type:#X} to {tmp_current_trigger_type:#X}."
                    )
                    .unwrap()
                }
                if rdh_cru.rdh3().detector_field != last_rdh_cru.rdh3().detector_field {
                    err_cnt += 1;
                    let tmp_current_detector_field = rdh_cru.rdh3().detector_field;
                    let tmp_last_detector_field = last_rdh_cru.rdh3().detector_field;
                    write!(
                        err_str,
                        "Detector field changed from {tmp_last_detector_field:#X} to {tmp_current_detector_field:#X}."
                    )
                    .unwrap()
                }
                if rdh_cru.rdh0().fee_id != last_rdh_cru.rdh0().fee_id {
                    err_cnt += 1;
                    let tmp_current_fee_id = rdh_cru.rdh0().fee_id.0;
                    let tmp_last_fee_id = last_rdh_cru.rdh0().fee_id.0;
                    write!(
                        err_str,
                        "FeeId changed from {tmp_last_fee_id:#X} to {tmp_current_fee_id:#X}."
                    )
                    .unwrap()
                }
            }
        }
        if err_cnt != 0 {
            Err(err_str)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RdhCruRunningChecker;
    use crate::input::prelude::test_data::*;
    use crate::input::prelude::*;
    use crate::input::rdh::rdh0::FeeId;
    use crate::input::rdh::rdh1::BcReserved;
    use crate::input::rdh::rdh_cru::{CruidDw, DataformatReserved};

    #[test]
    fn test_valid_rdh_crus() {
        let mut rdh_cru_checker = RdhCruRunningChecker::<RdhCru<V7>>::new();

        let rdh_1 = RdhCru::<V7>::load(&mut CORRECT_RDH_CRU_V7.to_byte_slice()).unwrap();
        let rdh_2 = RdhCru::<V7>::load(&mut CORRECT_RDH_CRU_V7_NEXT.to_byte_slice()).unwrap();
        let rdh_3_stop =
            RdhCru::<V7>::load(&mut CORRECT_RDH_CRU_V7_NEXT_NEXT_STOP.to_byte_slice()).unwrap();
        let res = rdh_cru_checker.check(&rdh_1);
        assert!(res.is_ok());
        let res = rdh_cru_checker.check(&rdh_2);
        assert!(res.is_ok());
        let res = rdh_cru_checker.check(&rdh_3_stop);
        assert!(res.is_ok());
    }

    #[test]
    fn test_invalid_first_second_is_same() {
        let mut rdh_cru_checker = RdhCruRunningChecker::<RdhCru<V7>>::new();

        let rdh_1 = RdhCru::<V7>::load(&mut CORRECT_RDH_CRU_V7.to_byte_slice()).unwrap();
        let rdh_2 = RdhCru::<V7>::load(&mut CORRECT_RDH_CRU_V7.to_byte_slice()).unwrap();
        let res = rdh_cru_checker.check(&rdh_1);
        assert!(res.is_ok());
        let res = rdh_cru_checker.check(&rdh_2);
        assert!(res.is_err());
        println!("{:?}", res);
    }

    #[test]
    fn test_valid_first_second_invalid_stop() {
        let mut rdh_cru_checker = RdhCruRunningChecker::<RdhCru<V7>>::new();

        let rdh_1 = RdhCru::<V7>::load(&mut CORRECT_RDH_CRU_V7.to_byte_slice()).unwrap();
        let rdh_2 = RdhCru::<V7>::load(&mut CORRECT_RDH_CRU_V7_NEXT.to_byte_slice()).unwrap();
        let rdh_3_stop =
            RdhCru::<V7>::load(&mut CORRECT_RDH_CRU_V7_NEXT_NEXT_STOP.to_byte_slice()).unwrap();
        let res = rdh_cru_checker.check(&rdh_1);
        assert!(res.is_ok());
        let res = rdh_cru_checker.check(&rdh_2);
        assert!(res.is_ok());
        let res = rdh_cru_checker.check(&rdh_3_stop);
        assert!(res.is_ok());
        let res = rdh_cru_checker.check(&rdh_3_stop);
        assert!(res.is_err());
        println!("{:?}", res);
    }

    #[test]
    fn test_invalid_first_is_stop() {
        let mut rdh_cru_checker = RdhCruRunningChecker::<RdhCru<V7>>::new();

        let rdh_1 =
            RdhCru::<V7>::load(&mut CORRECT_RDH_CRU_V7_NEXT_NEXT_STOP.to_byte_slice()).unwrap();
        let res = rdh_cru_checker.check(&rdh_1);
        assert!(res.is_err());
        println!("{:?}", res);
    }

    #[test]
    fn test_invalid_orbit_same_after_stop() {
        let mut rdh_cru_checker = RdhCruRunningChecker::<RdhCru<V7>>::new();

        let rdh_1 = RdhCru::<V7>::load(&mut CORRECT_RDH_CRU_V7.to_byte_slice()).unwrap();
        let rdh_2 = RdhCru::<V7>::load(&mut CORRECT_RDH_CRU_V7_NEXT.to_byte_slice()).unwrap();
        let rdh_3_stop =
            RdhCru::<V7>::load(&mut CORRECT_RDH_CRU_V7_NEXT_NEXT_STOP.to_byte_slice()).unwrap();
        let res = rdh_cru_checker.check(&rdh_1);
        assert!(res.is_ok());
        let res = rdh_cru_checker.check(&rdh_2);
        assert!(res.is_ok());
        let res = rdh_cru_checker.check(&rdh_3_stop);
        assert!(res.is_ok());
        let res = rdh_cru_checker.check(&rdh_1);
        assert!(res.is_err());
        println!("{:?}", res);
        assert!(res.unwrap_err().contains("Orbit"));
    }

    #[test]
    fn test_invalid_fields_not_same() {
        let mut rdh_cru_checker = RdhCruRunningChecker::<RdhCru<V7>>::new();

        let rdh_1 = RdhCru::<V7>::load(&mut CORRECT_RDH_CRU_V7.to_byte_slice()).unwrap();
        let rdh_2 = RdhCru::<V7>::load(&mut CORRECT_RDH_CRU_V7_NEXT.to_byte_slice()).unwrap();
        let rdh_3_different = RdhCru::<V7> {
            rdh0: Rdh0 {
                header_id: 0x7,
                header_size: 0x40,
                fee_id: FeeId(0xeffe),
                priority_bit: 0x0,
                system_id: 0x20,
                reserved0: 0,
            },
            offset_new_packet: 0x13E0,
            memory_size: 0x13E0,
            link_id: 0x0,
            packet_counter: 0x5,
            cruid_dw: CruidDw(0x0018),
            rdh1: Rdh1 {
                bc_reserved0: BcReserved(0x0),
                orbit: 0xcafebabe,
            },
            dataformat_reserved0: DataformatReserved(0x2),
            rdh2: Rdh2 {
                trigger_type: 0xbeef,
                pages_counter: 0x2,
                stop_bit: 0x0,
                reserved0: 0x0,
            },
            reserved1: 0x0,
            rdh3: Rdh3 {
                detector_field: 0xdead,
                par_bit: 0x0,
                reserved0: 0x0,
            },
            reserved2: 0x0,
            version: std::marker::PhantomData,
        };
        let res = rdh_cru_checker.check(&rdh_1);
        assert!(res.is_ok());
        let res = rdh_cru_checker.check(&rdh_2);
        assert!(res.is_ok());
        let res = rdh_cru_checker.check(&rdh_3_different);
        assert!(res.is_err());
        println!("{:?}", res);
        let err_str = res.unwrap_err();
        assert!(err_str.contains("Orbit"));
        assert!(err_str.contains("Trigger"));
        assert!(err_str.contains("Detector"));
        assert!(err_str.contains("FeeId"));
    }
}
