//! Miscellaneous utility functions
pub mod lib;

pub(crate) use {
    crate::{
        analyze::{
            validators::{
                its::{
                    self,
                    alpide::lane_alpide_frame_analyzer::LaneAlpideFrameAnalyzer,
                    cdp_running::CdpRunningValidator,
                    data_words::{
                        ib::IbDataWordValidator, ob::ObDataWordValidator, DATA_WORD_SANITY_CHECKER,
                    },
                    its_payload_fsm_cont::{self, ItsPayloadFsmContinuous},
                    lib::ItsPayloadWord,
                    status_word::StatusWordSanityChecker,
                },
                lib::preprocess_payload,
                link_validator::LinkValidator,
                rdh::RdhCruSanityValidator,
                rdh_running::RdhCruRunningChecker,
            },
            view::{
                self,
                lib::{calc_current_word_mem_pos, format_word_slice},
            },
        },
        config::{
            check::CheckModeArgs,
            custom_checks,
            inputoutput::{DataOutputFormat, DataOutputMode},
            prelude::*,
            Cfg,
        },
        stats::{
            self,
            stats_collector::{
                its_stats::alpide_stats::AlpideStats, rdh_stats::RdhStats, StatsCollector,
            },
            stats_report::report::{Report, StatSummary},
            StatType, SystemId,
        },
        words::{
            self,
            its::{
                layer_from_feeid,
                status_words::util::*,
                status_words::{cdw::Cdw, ddw::Ddw0, ihw::Ihw, tdh::Tdh, tdt::Tdt, StatusWord},
                stave_number_from_feeid,
            },
        },
    },
    alice_protocol_reader::{
        cdp_wrapper::cdp_array::CdpArray, prelude::*, rdh::rdh3::det_field_util,
    },
    byteorder::{ByteOrder, LittleEndian},
    clap::{
        builder::{
            styling::{AnsiColor, Effects},
            Styles,
        },
        Args, Subcommand,
    },
    crossbeam_channel, flume,
    indicatif::{ProgressBar, ProgressStyle},
    itertools::Itertools,
    owo_colors::OwoColorize,
    regex::Regex,
    ringbuffer::{ConstGenericRingBuffer, RingBuffer},
    serde::{Deserialize, Serialize},
    sm::sm,
    std::{
        error, fmt, fs, hint,
        io::{self, StdoutLock},
        marker::{self, PhantomData},
        mem,
        ops::RangeInclusive,
        path::{self, Path, PathBuf},
        process::ExitCode,
        slice::ChunksExact,
        str::{Chars, FromStr},
        sync::{
            atomic::{self, AtomicBool, Ordering},
            Arc, OnceLock,
        },
        thread::{self, Builder, JoinHandle},
        time::{Duration, Instant},
        vec::Drain,
    },
};
