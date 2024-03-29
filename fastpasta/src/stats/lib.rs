//! Contains utility related to collecting stats about the input data.

/// Module containing macros related to stats.
pub mod macros {
    /// Macro to generate the `validate_fields` function for a struct
    ///
    /// It's possible to do this without having to specify each field, although it would require a procedural macro which would require a whole new crate.
    #[macro_export]
    macro_rules! validate_fields {
    ($struct_name:ident, $($field:ident),*) => {
        fn validate_fields(&self, other: &$struct_name) -> Result<(), Vec<String>> {
            let mut errs = vec![];

            $(
                if self.$field != other.$field {
                    errs.push(format!("{field_name} mismatch! expected: {other_val:?}, got: {this_val:?}",
                        field_name = stringify!($field),
                        other_val = other.$field, this_val = self.$field
                        )
                    );
                }
            )*

            if errs.is_empty() {
                Ok(())
            } else {
                Err(errs)
            }
        }
    };
}
}

#[cfg(test)]
mod tests {
    use crate::stats::{collect_its_stats, collect_system_specific_stats, StatType, SystemId};
    use alice_protocol_reader::prelude::*;

    #[test]
    fn test_collect_its_stats() {
        let (stats_sender, stats_receiver) = flume::unbounded::<StatType>();
        let rdh = alice_protocol_reader::prelude::test_data::CORRECT_RDH_CRU_V7;

        let expect_layer = crate::words::its::layer_from_feeid(rdh.fee_id());
        let expect_stave = crate::words::its::stave_number_from_feeid(rdh.fee_id());

        collect_its_stats(&rdh, &stats_sender);

        let stats = stats_receiver.recv().unwrap();

        match stats {
            StatType::LayerStaveSeen { layer, stave } => {
                assert_eq!(layer, expect_layer);
                assert_eq!(stave, expect_stave);
            }
            _ => panic!("Wrong stat type received"),
        }
    }

    #[test]
    fn test_collect_system_specific_stats() {
        let (stats_sender, stats_receiver) = flume::unbounded::<StatType>();
        let mut system_id = None;

        let rdh = alice_protocol_reader::prelude::test_data::CORRECT_RDH_CRU_V6;
        let expect_layer = crate::words::its::layer_from_feeid(rdh.fee_id());
        let expect_stave = crate::words::its::stave_number_from_feeid(rdh.fee_id());

        collect_system_specific_stats(&rdh, &mut system_id, &stats_sender).unwrap();

        let stats = stats_receiver.recv().unwrap();

        match stats {
            StatType::LayerStaveSeen { layer, stave } => {
                assert_eq!(layer, expect_layer);
                assert_eq!(stave, expect_stave);
            }
            _ => panic!("Wrong stat type received"),
        }
    }

    #[test]
    fn test_system_id_from_system_id() {
        let system_id = SystemId::from_system_id(32).unwrap();
        assert_eq!(system_id, SystemId::ITS);
        let as_string = format!("{system_id}");
        assert_eq!(as_string, "ITS");
    }

    #[test]
    fn test_system_id_enums_all() {
        let valid_system_ids: [u8; 20] = [
            3, 4, 5, 6, 7, 8, 10, 15, 17, 18, 19, 32, 33, 34, 35, 36, 37, 38, 39, 255,
        ];
        for id in 0..=255 {
            let system_id = SystemId::from_system_id(id);
            if valid_system_ids.contains(&id) {
                assert!(system_id.is_ok());
                let to_str = system_id.unwrap().to_string();
                assert!(!to_str.is_empty());
            } else {
                assert!(system_id.is_err());
                let to_str = system_id.unwrap_err().to_string();
                assert_eq!(to_str, format!("Unknown system ID {id}"));
            }
        }
    }

    #[test]
    fn test_all_stattype_enums() {
        let fatal = StatType::Fatal("Test fatal error".to_string().into());
        let error = StatType::Error("Test error".to_string().into());
        let run_trig_type =
            StatType::RunTriggerType((1, "Test run trigger type".to_string().into()));
        let sys_id = StatType::SystemId(SystemId::ITS);
        let rdh_seen = StatType::RDHSeen(1);
        let rdh_filtered = StatType::RDHFiltered(1);
        let layer_stave_seen = StatType::LayerStaveSeen { layer: 1, stave: 1 };
        let mut stat_type_vec = vec![
            fatal,
            error,
            run_trig_type,
            sys_id,
            rdh_seen,
            rdh_filtered,
            layer_stave_seen,
        ];

        stat_type_vec.push(StatType::PayloadSize(1));
        stat_type_vec.push(StatType::LinksObserved(0));
        stat_type_vec.push(StatType::RdhVersion(1));
        stat_type_vec.push(StatType::DataFormat(1));

        for stat_type in stat_type_vec {
            // Test to_string() method
            let to_str = stat_type.to_string();
            assert!(!to_str.is_empty());
        }
    }
}
