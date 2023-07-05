use super::rdh_stats::RdhStats;
use crate::util::config::custom_checks::CustomChecksOpt;

pub fn validate_custom_stats(
    custom_checks: &impl CustomChecksOpt,
    rdh_stats: &RdhStats,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if let Some(cdps) = custom_checks.cdps() {
        if rdh_stats.rdhs_seen() != cdps as u64 {
            errors.push(format!(
                "[E99] Expected {expected_cdps} CDPS, but found {observed_cdps}",
                expected_cdps = cdps,
                observed_cdps = rdh_stats.rdhs_seen()
            ));
        }
    }
    if let Some(triggers_sent) = custom_checks.triggers_pht() {
        todo!();
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
