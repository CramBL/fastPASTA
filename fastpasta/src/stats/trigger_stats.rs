#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::fmt::Display;
#[derive(Debug, PartialEq, Clone, Default, Deserialize, Serialize)]
pub struct TriggerStats {
    orbit: u32,
    // [1] Heart Beat flag
    hb: u32,
    // [2] Heart Beat Reject flag
    hbr: u32,
    // [3]  Health Check
    hc: u32,
    // [4]  Physics Trigger
    pht: u32,
    // [5]  Pre Pulse for calibration
    pp: u32,
    // [6]  Calibration Trigger
    cal: u32,
    // [7]  Start of Triggered Data
    sot: u32,
    // [8]  End of Triggered Data
    eot: u32,
    // [9]  Start of Continuous Data
    soc: u32,
    // [10] End of Continuous Data
    eoc: u32,
    // [11] Time Frame delimiter
    tf: u32,
    // [12] Front End reset
    fe_rst: u32,
    // Run Type; 1 = Continuous, 0 = Triggered
    rt: u32,
    // [14]  Running State; 1 = Running
    rs: u32,
    // [15..26] Spare
    // [27]  LHC about gap 1
    lhc_gap1: u32,
    // [28]  LHC about gap 2
    lhc_gap2: u32,
    // [29]  TPC synchronization/ITS reset
    tpc_sync: u32,
    // [30]  On request reset
    tpc_rst: u32,
    // [31]  TOF special trigger
    tof: u32,
}

impl TriggerStats {
    pub fn collect_stats(&mut self, trigger: u32) {
        // Checks the trigger for each bit and increments the corresponding counters
        if trigger & 0b0000_0000_0000_0000_0000_0000_0000_0001 != 0 {
            self.orbit += 1;
        }
        if trigger & 0b0000_0000_0000_0000_0000_0000_0000_0010 != 0 {
            self.hb += 1;
        }
        if trigger & 0b0000_0000_0000_0000_0000_0000_0000_0100 != 0 {
            self.hbr += 1;
        }
        if trigger & 0b0000_0000_0000_0000_0000_0000_0000_1000 != 0 {
            self.hc += 1;
        }
        if trigger & 0b0000_0000_0000_0000_0000_0000_0001_0000 != 0 {
            self.pht += 1;
        }
        if trigger & 0b0000_0000_0000_0000_0000_0000_0010_0000 != 0 {
            self.pp += 1;
        }
        if trigger & 0b0000_0000_0000_0000_0000_0000_0100_0000 != 0 {
            self.cal += 1;
        }
        if trigger & 0b0000_0000_0000_0000_0000_0000_1000_0000 != 0 {
            self.sot += 1;
        }
        if trigger & 0b0000_0000_0000_0000_0000_0001_0000_0000 != 0 {
            self.eot += 1;
        }
        if trigger & 0b0000_0000_0000_0000_0000_0010_0000_0000 != 0 {
            self.soc += 1;
        }
        if trigger & 0b0000_0000_0000_0000_0000_0100_0000_0000 != 0 {
            self.eoc += 1;
        }
        if trigger & 0b0000_0000_0000_0000_0000_1000_0000_0000 != 0 {
            self.tf += 1;
        }
        if trigger & 0b0000_0000_0000_0000_0001_0000_0000_0000 != 0 {
            self.fe_rst += 1;
        }
        if trigger & 0b0000_0000_0000_0000_0010_0000_0000_0000 != 0 {
            self.rt += 1;
        }
        if trigger & 0b0000_0000_0000_0000_0100_0000_0000_0000 != 0 {
            self.rs += 1;
        }
        if trigger & 0b0000_1000_0000_0000_0000_0000_0000_0000 != 0 {
            self.lhc_gap1 += 1;
        }
        if trigger & 0b0001_0000_0000_0000_0000_0000_0000_0000 != 0 {
            self.lhc_gap2 += 1;
        }
        if trigger & 0b0010_0000_0000_0000_0000_0000_0000_0000 != 0 {
            self.tpc_sync += 1;
        }
        if trigger & 0b0100_0000_0000_0000_0000_0000_0000_0000 != 0 {
            self.tpc_rst += 1;
        }
        if trigger & 0b1000_0000_0000_0000_0000_0000_0000_0000 != 0 {
            self.tof += 1;
        }
    }

    pub fn orbit(&self) -> u32 {
        self.orbit
    }

    pub fn hb(&self) -> u32 {
        self.hb
    }

    pub fn hbr(&self) -> u32 {
        self.hbr
    }

    pub fn hc(&self) -> u32 {
        self.hc
    }

    pub fn pht(&self) -> u32 {
        self.pht
    }

    pub fn pp(&self) -> u32 {
        self.pp
    }

    pub fn cal(&self) -> u32 {
        self.cal
    }

    pub fn sot(&self) -> u32 {
        self.sot
    }

    pub fn eot(&self) -> u32 {
        self.eot
    }

    pub fn soc(&self) -> u32 {
        self.soc
    }

    pub fn eoc(&self) -> u32 {
        self.eoc
    }

    pub fn tf(&self) -> u32 {
        self.tf
    }

    pub fn fe_rst(&self) -> u32 {
        self.fe_rst
    }

    pub fn rt(&self) -> u32 {
        self.rt
    }

    pub fn rs(&self) -> u32 {
        self.rs
    }

    pub fn lhc_gap1(&self) -> u32 {
        self.lhc_gap1
    }

    pub fn lhc_gap2(&self) -> u32 {
        self.lhc_gap2
    }

    pub fn tpc_sync(&self) -> u32 {
        self.tpc_sync
    }

    pub fn tpc_rst(&self) -> u32 {
        self.tpc_rst
    }

    pub fn tof(&self) -> u32 {
        self.tof
    }
}

impl Display for TriggerStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Trigger statistics:")?;
        writeln!(f, "  Orbit:    {:>6}", self.orbit)?;
        writeln!(f, "  HB:       {:>6}", self.hb)?;
        writeln!(f, "  HBr:      {:>6}", self.hbr)?;
        writeln!(f, "  HC:       {:>6}", self.hc)?;
        writeln!(f, "  PhT:      {:>6}", self.pht)?;
        writeln!(f, "  PP:       {:>6}", self.pp)?;
        writeln!(f, "  CAL:      {:>6}", self.cal)?;
        writeln!(f, "  SOT:      {:>6}", self.sot)?;
        writeln!(f, "  EOT:      {:>6}", self.eot)?;
        writeln!(f, "  SOC:      {:>6}", self.soc)?;
        writeln!(f, "  EOC:      {:>6}", self.eoc)?;
        writeln!(f, "  TF:       {:>6}", self.tf)?;
        writeln!(f, "  FE_rst:   {:>6}", self.fe_rst)?;
        writeln!(f, "  RT:       {:>6}", self.rt)?;
        writeln!(f, "  RS:       {:>6}", self.rs)?;
        writeln!(f, "  LHC_gap1: {:>6}", self.lhc_gap1)?;
        writeln!(f, "  LHC_gap2: {:>6}", self.lhc_gap2)?;
        writeln!(f, "  TPC_sync: {:>6}", self.tpc_sync)?;
        writeln!(f, "  TPC_rst:  {:>6}", self.tpc_rst)?;
        writeln!(f, "  TOF:      {:>6}", self.tof)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_serde_consistency() {
        // Test JSON and TOML serialization and deserialization
        let mut trigger_stats = TriggerStats::default();
        trigger_stats.collect_stats(0b0000_0000_0000_0000_0000_0000_0000_0001);
        trigger_stats.collect_stats(0b0000_0000_0000_0000_0000_0000_0000_0010);
        trigger_stats.collect_stats(0b0000_0000_0000_0000_0000_0000_0000_0100);
        trigger_stats.collect_stats(0b0000_0000_0000_0000_0000_0000_0000_1000);
        trigger_stats.collect_stats(0b0000_0000_0000_0000_0000_0000_0001_0000);
        trigger_stats.collect_stats(0b0000_0000_0000_0000_0000_0000_0010_0000);
        trigger_stats.collect_stats(0b0000_0000_0000_0000_0000_0000_0100_0000);
        trigger_stats.collect_stats(0b0000_0000_0000_0000_0000_0000_1000_0000);
        trigger_stats.collect_stats(0b0000_0000_0000_0000_0000_0001_0000_0000);
        trigger_stats.collect_stats(0b0000_0000_0000_0000_0000_0010_0000_0000);
        trigger_stats.collect_stats(0b0000_0000_0000_0000_0000_0100_0000_0000);
        trigger_stats.collect_stats(0b0000_0000_0000_0000_0000_1000_0000_0000);
        trigger_stats.collect_stats(0b0000_0000_0000_0000_0001_0000_0000_0000);
        trigger_stats.collect_stats(0b0000_0000_0000_0000_0010_0000_0000_0000);
        trigger_stats.collect_stats(0b0000_0000_0000_0000_0100_0000_0000_0000);
        trigger_stats.collect_stats(0b0000_0000_0000_0000_1000_0000_0000_0000);
        trigger_stats.collect_stats(0b0000_0000_0000_0001_0000_0000_0000_0000);
        trigger_stats.collect_stats(0b0000_0000_0000_0010_0000_0000_0000_0000);
        trigger_stats.collect_stats(0b0000_0000_0000_0100_0000_0000_0000_0000);
        trigger_stats.collect_stats(0b0000_0000_0000_1000_0000_0000_0000_0000);
        trigger_stats.collect_stats(0b0000_0000_0001_0000_0000_0000_0000_0000);
        trigger_stats.collect_stats(0b0000_0000_0010_0000_0000_0000_0000_0000);
        trigger_stats.collect_stats(0b0000_0000_0100_0000_0000_0000_0000_0000);
        trigger_stats.collect_stats(0b0000_0000_1000_0000_0000_0000_0000_0000);
        trigger_stats.collect_stats(0b0000_0001_0000_0000_0000_0000_0000_0000);
        trigger_stats.collect_stats(0b0000_0010_0000_0000_0000_0000_0000_0000);
        trigger_stats.collect_stats(0b0000_0100_0000_0000_0000_0000_0000_0000);
        trigger_stats.collect_stats(0b0000_1000_0000_0000_0000_0000_0000_0000);
        trigger_stats.collect_stats(0b0001_0000_0000_0000_0000_0000_0000_0000);
        trigger_stats.collect_stats(0b0010_0000_0000_0000_0000_0000_0000_0000);
        trigger_stats.collect_stats(0b0100_0000_0000_0000_0000_0000_0000_0000);
        trigger_stats.collect_stats(0b1000_0000_0000_0000_0000_0000_0000_0000);

        // JSON
        let trigger_stats_ser_json = serde_json::to_string(&trigger_stats).unwrap();
        let trigger_stats_de_json: TriggerStats =
            serde_json::from_str(&trigger_stats_ser_json).unwrap();
        assert_eq!(trigger_stats, trigger_stats_de_json);

        // TOML
        let trigger_stats_ser_toml = toml::to_string(&trigger_stats).unwrap();
        let trigger_stats_de_toml: TriggerStats = toml::from_str(&trigger_stats_ser_toml).unwrap();
        assert_eq!(trigger_stats, trigger_stats_de_toml);

        // Display
        println!("{}", trigger_stats_ser_json);
        println!("{}", trigger_stats_ser_toml);

        // Make sure the counters are correct
        assert_eq!(trigger_stats.orbit(), 1);
        assert_eq!(trigger_stats.hb(), 1);
        assert_eq!(trigger_stats.hbr(), 1);
        assert_eq!(trigger_stats.hc(), 1);
        assert_eq!(trigger_stats.pht(), 1);
        assert_eq!(trigger_stats.pp(), 1);
        assert_eq!(trigger_stats.cal(), 1);
        assert_eq!(trigger_stats.sot(), 1);
        assert_eq!(trigger_stats.eot(), 1);
        assert_eq!(trigger_stats.soc(), 1);
        assert_eq!(trigger_stats.eoc(), 1);
        assert_eq!(trigger_stats.tf(), 1);
        assert_eq!(trigger_stats.fe_rst(), 1);
        assert_eq!(trigger_stats.rt(), 1);
        assert_eq!(trigger_stats.rs(), 1);
        assert_eq!(trigger_stats.lhc_gap1(), 1);
        assert_eq!(trigger_stats.lhc_gap2(), 1);
        assert_eq!(trigger_stats.tpc_sync(), 1);
        assert_eq!(trigger_stats.tpc_rst(), 1);
        assert_eq!(trigger_stats.tof(), 1);
    }
}
