use crate::words::{
    lib::RDH,
    rdh::{Rdh1, Rdh2},
};
use std::fmt::Write;
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
        let mut rdh_errors: Vec<String> = vec![];
        let mut err_cnt: u8 = 0;

        if let Err(e) = self.check_stop_bit_and_page_counter(rdh.rdh2()) {
            err_cnt += 1;
            rdh_errors.push(e);
        };

        if let Err(e) = self.check_orbit_counter_changes(rdh.rdh1()) {
            err_cnt += 1;
            rdh_errors.push(e);
        };

        if err_cnt != 0 {
            rdh_errors.into_iter().for_each(|e| {
                err_str.push_str(&e);
            });
            return Err(err_str);
        }

        self.last_rdh_cru = Some(T::load(&mut rdh.to_byte_slice()).unwrap());

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
        let mut err_str = String::from("RDH2 check stop_bit & page_counter failed: ");
        let mut err_cnt: u8 = 0;
        match rdh2.stop_bit {
            0 => {
                if rdh2.pages_counter != self.expect_pages_counter {
                    err_cnt += 1;
                    let tmp = rdh2.pages_counter;
                    write!(
                        err_str,
                        "pages_counter = {tmp} expected: {}",
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
                        "pages_counter = {tmp} expected: {}",
                        self.expect_pages_counter
                    )
                    .unwrap();
                }
                self.expect_pages_counter = 0;
            }
            _ => {
                err_cnt += 1;
                let tmp = rdh2.stop_bit;
                write!(err_str, "stop_bit = {tmp}").unwrap();
            }
        };

        if err_cnt != 0 {
            return Err(err_str.to_owned());
        }

        Ok(())
    }

    #[inline]
    fn check_orbit_counter_changes(&self, rdh1: &Rdh1) -> Result<(), String> {
        if let Some(last_rdh_cru) = &self.last_rdh_cru {
            let current_orbit = rdh1.orbit;
            if last_rdh_cru.rdh1().orbit == current_orbit {
                return Err(format!("Orbit same as previous {current_orbit} "));
            }
        }
        Ok(())
    }
}
