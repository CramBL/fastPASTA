use criterion::{black_box, BenchmarkId, Criterion};

#[derive(Debug, PartialEq, Clone, Copy, Default)]
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
    pub fn collect_stats_current(&mut self, trigger: u32) {
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

    pub fn collect_stats_current_branchless(&mut self, trigger: u32) {
        // Checks the trigger for each bit and increments the corresponding counters

        self.orbit += (trigger & 0b0000_0000_0000_0000_0000_0000_0000_0001 != 0) as u32;
        self.hb += (trigger & 0b0000_0000_0000_0000_0000_0000_0000_0010 != 0) as u32;
        self.hbr += (trigger & 0b0000_0000_0000_0000_0000_0000_0000_0100 != 0) as u32;
        self.hc += (trigger & 0b0000_0000_0000_0000_0000_0000_0000_1000 != 0) as u32;
        self.pht += (trigger & 0b0000_0000_0000_0000_0000_0000_0001_0000 != 0) as u32;
        self.pp += (trigger & 0b0000_0000_0000_0000_0000_0000_0010_0000 != 0) as u32;
        self.cal += (trigger & 0b0000_0000_0000_0000_0000_0000_0100_0000 != 0) as u32;
        self.sot += (trigger & 0b0000_0000_0000_0000_0000_0000_1000_0000 != 0) as u32;
        self.eot += (trigger & 0b0000_0000_0000_0000_0000_0001_0000_0000 != 0) as u32;
        self.soc += (trigger & 0b0000_0000_0000_0000_0000_0010_0000_0000 != 0) as u32;
        self.eoc += (trigger & 0b0000_0000_0000_0000_0000_0100_0000_0000 != 0) as u32;
        self.tf += (trigger & 0b0000_0000_0000_0000_0000_1000_0000_0000 != 0) as u32;
        self.fe_rst += (trigger & 0b0000_0000_0000_0000_0001_0000_0000_0000 != 0) as u32;
        self.rt += (trigger & 0b0000_0000_0000_0000_0010_0000_0000_0000 != 0) as u32;
        self.rs += (trigger & 0b0000_0000_0000_0000_0100_0000_0000_0000 != 0) as u32;
        self.lhc_gap1 += (trigger & 0b0000_1000_0000_0000_0000_0000_0000_0000 != 0) as u32;
        self.lhc_gap2 += (trigger & 0b0001_0000_0000_0000_0000_0000_0000_0000 != 0) as u32;
        self.tpc_sync += (trigger & 0b0010_0000_0000_0000_0000_0000_0000_0000 != 0) as u32;
        self.tpc_rst += (trigger & 0b0100_0000_0000_0000_0000_0000_0000_0000 != 0) as u32;
        self.tof += (trigger & 0b1000_0000_0000_0000_0000_0000_0000_0000 != 0) as u32;
    }
}

pub fn bench_collect_trigger_stats(c: &mut Criterion) {
    let mut group = c.benchmark_group("collect_trigger_stats");

    let mut trig_collector = TriggerStats::default();

    for i in [0x6a03, 0x6003, 0x6803, 0x4813].iter() {
        group.bench_with_input(BenchmarkId::new("Current", i.to_string()), i, |b, i| {
            b.iter(|| trig_collector.collect_stats_current(black_box(*i)));
        });
        group.bench_with_input(
            BenchmarkId::new("Current_branchless", i.to_string()),
            i,
            |b, i| {
                b.iter(|| trig_collector.collect_stats_current_branchless(black_box(*i)));
            },
        );
    }
    group.finish();
}
