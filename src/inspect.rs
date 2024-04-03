//! Inspecting a busfile for statistics
//!
//! just like `bustools inspect`
use bustools::{
    io::BusReader,
    iterators::{CbUmiGroupIterator, CellGroupIterator},
};

#[derive(Debug, Eq, PartialEq)]
struct BusStatistics {
    cb_len: usize,
    umi_len: usize,
    nrecords: usize,
    nreads: usize,
    n_cells: usize,
    n_cbumi: usize,
}

fn _inspect(busfile: &str) -> BusStatistics {
    let n_cbumi = BusReader::new(busfile).groupby_cbumi().count();
    let n_cells = BusReader::new(busfile).groupby_cb().count();

    let bf = BusReader::new(busfile);
    let params = bf.get_params();
    let cb_len = params.cb_len as usize;
    let umi_len = params.umi_len as usize;

    let mut nreads = 0;
    let mut nrecords = 0;

    let bus = BusReader::new(busfile);
    for r in bus {
        nrecords += 1;
        nreads += r.COUNT as usize
    }

    // match BusReader::new(busfile) {
    //     BusReader::Plain(reader) => {reader.get_bus_header()}
    // }

    BusStatistics {cb_len,umi_len, nrecords, nreads, n_cells, n_cbumi }
}

/// Inspect a busfile, counting number of reads, records, cb-umi combinations and cell-barcodes
/// # Example
/// ```rust, no_run
/// # use bustools::inspect::inspect;
/// inspect("somefile.bus")
/// ```
pub fn inspect(busfile: &str) {
    let stats = _inspect(busfile);
    println!("CB: {} BP, UMI: {} BP", stats.cb_len, stats.umi_len);
    println!("{} BUS records", stats.nrecords);
    println!("{} reads", stats.nreads);
    println!("{} cell-barcodes", stats.n_cells);
    println!("{} CB-UMIs", stats.n_cbumi);
}

#[cfg(test)]
mod testing {
    use super::{BusStatistics, _inspect};
    use bustools::io::{setup_busfile, BusRecord};

    #[test]
    fn test_inspect() {
        let r1 = BusRecord { CB: 0, UMI: 2, EC: 0, COUNT: 12, FLAG: 0 };
        let r2 = BusRecord { CB: 0, UMI: 21, EC: 1, COUNT: 2, FLAG: 0 };
        let r3 = BusRecord { CB: 1, UMI: 2, EC: 0, COUNT: 12, FLAG: 0 };
        let r4 = BusRecord { CB: 2, UMI: 1, EC: 1, COUNT: 2, FLAG: 0 };
        let r5 = BusRecord { CB: 2, UMI: 21, EC: 1, COUNT: 2, FLAG: 0 };
        let r6 = BusRecord { CB: 3, UMI: 1, EC: 1, COUNT: 2, FLAG: 0 };
        let r7 = BusRecord { CB: 3, UMI: 1, EC: 10, COUNT: 2, FLAG: 0 };

        let records = vec![
            r1.clone(),
            r2.clone(),
            r3.clone(),
            r4.clone(),
            r5.clone(),
            r6.clone(),
            r7.clone(),
        ];
        // let records = vec![r1,r2,r3,r4,r5, r6].to_vec();

        let (busname, _dir) = setup_busfile(&records);

        let r = _inspect(&busname);
        assert_eq!(
            r,
            BusStatistics {cb_len: 16, umi_len: 12, nrecords: 7, nreads: 34, n_cells: 4, n_cbumi: 6 }
        );
    }
}
