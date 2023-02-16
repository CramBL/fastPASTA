use fastpasta::data_words::rdh::RdhCRUv7;
use fastpasta::{setup_buffered_reading, FilterLink, GbtWord};

pub enum SeekError {
    EOF,
}

pub fn main() -> std::io::Result<()> {
    let opt = <fastpasta::Opt as structopt::StructOpt>::from_args();
    println!("{:#?}", opt);
    let mut stats = fastpasta::Stats::new();

    let mut buf_reader = setup_buffered_reading(&opt);
    let mut file_tracker = fastpasta::FilePosTracker::new();

    // 1. Create reader
    //      - Open file in read only mode
    //      - Wrap in BufReader
    //      - Track file position
    //      - reads data through struct interface + buffer
    // 2. Read into in reasonably sized buffer
    // 3. Pass buffer to checker and read another chunk
    // 4. Checker verifies received buffered chunk (big checks -> multi-threading)
    //                Not valid -> Print error and abort
    //                Valid     -> Pass chunk to writer
    // 5. Writer writes chunk to file OR stdout

    let mut rdhs: Vec<RdhCRUv7> = vec![];

    let mut running_rdh_checker = fastpasta::validators::rdh::RdhCruv7RunningChecker::new();
    // TODO: Single entry point to a filtering mechanism if filtering is enabled
    let mut filter_link: Option<FilterLink> = None;

    if opt.filter_link().is_some() {
        println!("Filtering link: {:?}", opt.filter_link().unwrap());
        filter_link = Some(FilterLink::new(&opt, 1024));
    }

    loop {
        let tmp_rdh = match RdhCRUv7::load(&mut buf_reader) {
            Ok(rdh) => rdh,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                print!("EOF reached! ");
                break;
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
        };
        if opt.sanity_checks() {
            sanity_validation(&tmp_rdh);
        }
        // RDH CHECK: There is always page 0 + minimum page 1 + stop flag
        match running_rdh_checker.check(&tmp_rdh) {
            Ok(_) => (),
            Err(e) => {
                println!("RDH check failed: {}", e);
                println!("Last RDH:");
                running_rdh_checker.last_rdh2.unwrap().print();
                println!("Current RDH:");
                tmp_rdh.print();
                stats.print();

                break;
            }
        }
        // Seen links stats:
        if stats.links_observed.contains(&tmp_rdh.link_id) == false {
            stats.links_observed.push(tmp_rdh.link_id);
        }

        if filter_link.is_some() {
            let offset = tmp_rdh.offset_new_packet;
            if filter_link
                .as_mut()
                .unwrap()
                .filter_link(&mut buf_reader, tmp_rdh)
                == false
            {
                buf_reader
                    .seek_relative(file_tracker.next(offset as u64))
                    .expect("Error seeking");
            }
        } else {
            buf_reader
                .seek_relative(file_tracker.next(tmp_rdh.offset_new_packet as u64))
                .expect("Error seeking");
            if opt.dump_rhds() {
                if opt.output().is_some() {
                    rdhs.push(tmp_rdh);
                } else {
                    println!("{:?}", tmp_rdh);
                }
            }
        }

        stats.total_rdhs += 1;
    }
    println!("Vec size: {}", rdhs.len());

    if filter_link.is_some() {
        filter_link.unwrap().print_stats();
    }

    debug_assert!(running_rdh_checker.expect_pages_counter == 0);
    stats.print();
    Ok(())
}

pub fn sanity_validation(rdh: &RdhCRUv7) {
    let rdh_validator = fastpasta::validators::rdh::RDH_CRU_V7_VALIDATOR;
    match rdh_validator.sanity_check(&rdh) {
        Ok(_) => (),
        Err(e) => {
            println!("Sanity check failed: {}", e);
        }
    }
}
