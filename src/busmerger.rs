//! Filtering/Merging busfiles on CB/UMI overlap
use bustools::{
    bus_multi::CellUmiIteratorMulti,
    io::{BusWriter, BusParams},
};
use std::collections::HashMap;

/// will extract all busrecords that appear in both inputs and write them to the respective outputs
///
/// there'll be two output files, each contining the shared reads from the respective input file
/// ## Parameters:
/// * busfile1: first input
/// * busfile2: 2nd input
/// * outfile1: 1st output: will contain all CB/UMI that also appear in busfile2 (not the records itself (EC,COUNT) can be different from busfile2)
/// * outfile2: 2st output: will contain all CB/UMI that also appear in busfile1 (not the records itself (EC,COUNT) can be different from busfile2)
pub fn merge_busfiles_on_overlap(busfile1: &str, busfile2: &str, outfile1: &str, outfile2: &str) {
    //
    let h: HashMap<String, String> = HashMap::from([
        ("f1".to_string(), busfile1.to_string()),
        ("f2".to_string(), busfile2.to_string()),
    ]);

    let params = BusParams {cb_len: 16, umi_len: 12};
    let mut writers: HashMap<String, BusWriter> = HashMap::from([
        (
            "f1".to_string(),
            BusWriter::new(outfile1, params.clone()),
        ),
        (
            "f2".to_string(),
            BusWriter::new(outfile2, params),
        ),
    ]);

    let cbumi_merge_iter = CellUmiIteratorMulti::new(&h);

    for (_cbumi, record_map) in cbumi_merge_iter {
        // if the CB/UMI is present in both files, write
        if record_map.len() == 2 {
            for (name, records) in record_map {
                let w1 = writers.get_mut(&name).unwrap();
                w1.write_records(&records)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bustools::io::{setup_busfile, BusReader, BusRecord};

    fn get_records(fname: &str) -> Vec<BusRecord> {
        let reader = BusReader::new(fname);
        let records: Vec<BusRecord> = reader.into_iter().collect();
        records
    }

    #[test]
    fn test_merge() {
        let r1 = BusRecord { CB: 0, UMI: 21, EC: 0, COUNT: 2, FLAG: 0 };
        let r2 = BusRecord { CB: 1, UMI: 2, EC: 0, COUNT: 12, FLAG: 0 };
        let r3 = BusRecord { CB: 1, UMI: 3, EC: 0, COUNT: 2, FLAG: 0 };
        let r4 = BusRecord { CB: 3, UMI: 0, EC: 0, COUNT: 2, FLAG: 0 };
        let r5 = BusRecord { CB: 3, UMI: 0, EC: 1, COUNT: 2, FLAG: 0 };

        let v1 = vec![r1.clone(), r2.clone(), r3.clone(), r4.clone(), r5.clone()];

        let s2 = BusRecord { CB: 1, UMI: 2, EC: 1, COUNT: 12, FLAG: 0 };
        let s3 = BusRecord { CB: 2, UMI: 3, EC: 1, COUNT: 2, FLAG: 0 };
        let s4 = BusRecord { CB: 3, UMI: 0, EC: 1, COUNT: 2, FLAG: 0 };

        let v2 = vec![s2.clone(), s3.clone(), s4.clone()];

        // let input1 = "/tmp/merge1.bus";
        // let input2 = "/tmp/merge2.bus";

        let (input1, _dir1) = setup_busfile(&v1); //input1
        let (input2, _dir2) = setup_busfile(&v2); // input2

        let output1_path = _dir1.path().join("merge1_out.bus");
        let output1 = output1_path.to_str().unwrap();
        let output2_path = _dir2.path().join("merge2_out.bus");
        let output2 = output2_path.to_str().unwrap();

        merge_busfiles_on_overlap(&input1, &input2, output1, output2);

        assert_eq!(get_records(output1), vec![r2, r4, r5]);
        assert_eq!(get_records(output2), vec![s2, s4]);
    }
}
