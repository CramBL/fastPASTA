#![allow(dead_code)]
#[derive(Debug, Clone, Default)]
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
