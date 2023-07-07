use super::rdh_stats::RdhStats;
use crate::util::config::custom_checks::CustomChecksOpt;

pub fn validate_custom_stats(
    custom_checks: &'static impl CustomChecksOpt,
    rdh_stats: &RdhStats,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if let Some(cdps) = custom_checks.cdps() {
        if rdh_stats.rdhs_seen() != cdps as u64 {
            errors.push(format!(
                "[E99] Expected {expected_cdps} CDPs, but found {observed_cdps}",
                expected_cdps = cdps,
                observed_cdps = rdh_stats.rdhs_seen()
            ));
        }
    }
    if let Some(expect_triggers_pht) = custom_checks.triggers_pht() {
        if rdh_stats.trigger_stats().pht() != expect_triggers_pht {
            errors.push(format!(
                "[E99] Expected {expected_triggers_pht} PhT triggers, but found {observed_triggers_pht}.\n{triggers_stats}",
                expected_triggers_pht = expect_triggers_pht,
                observed_triggers_pht = rdh_stats.trigger_stats().pht(),
                triggers_stats = rdh_stats.trigger_stats()
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
