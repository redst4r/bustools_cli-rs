//! `bustools sort` code. Sorts busfiles by CB/UMI/EC
//!
//! # Merging records
//! Note that this not only sorts records according to CB/UMI/EC,
//! but also merges records with the same CB/UMI/EC/FLAG (adding up their counts)
//!
#![deny(missing_docs)]
use bustools::{
    io::{BusReader, BusRecord, BusWriter},
    iterators::CbUmiGroupIterator,
    merger::MultiIterator,
};
use itertools::Itertools;
use std::collections::{BTreeMap, HashMap};
use tempfile::tempdir;

/// sorts/inserts an Iterator over records into a BTreeMap,
/// (CB,UMI,EC, FLAG) -> records
/// This effectively sorts the records in memory and aggregates records with the same CB/UMI/EC/FLAG
fn sort_into_btree<I: Iterator<Item = BusRecord>>(
    iterator: I,
) -> BTreeMap<(u64, u64, u32, u32), BusRecord> {
    let mut in_mem_sort: BTreeMap<(u64, u64, u32, u32), BusRecord> = BTreeMap::new();

    for record in iterator {
        if let Some(r) = in_mem_sort.get_mut(&(record.CB, record.UMI, record.EC, record.FLAG)) {
            r.COUNT += record.COUNT
        }
        else {
            in_mem_sort.insert((record.CB, record.UMI, record.EC, record.FLAG), record);
        }
    }
    in_mem_sort
}

/// Sort a busfile (via CB/UMI/EC) in memory, using BTreeMap's internal sorting!
/// This gets quite bad for larger files!
///
/// # Parameters
/// * `busfile`: file to be sorted in memory
/// * `outfile`: file to be sorted into
fn sort_in_memory(busfile: &str, outfile: &str) {
    let reader = BusReader::new(busfile);
    let header = reader.bus_header.clone();

    let in_mem_sort = sort_into_btree(reader);

    // write out
    let mut writer = BusWriter::new(outfile, header);
    for (_cbumi, record) in in_mem_sort {
        writer.write_record(&record);
    }
}

/// Merges records (CB/UMI/EC) that got split over different chunks
fn merge_chunks(record_dict: HashMap<String, Vec<BusRecord>>) -> Vec<BusRecord>{
    let records_from_all_chunks = record_dict.into_values().flatten();
    let btree_sorted: Vec<BusRecord> = sort_into_btree(records_from_all_chunks).into_values().collect();
    btree_sorted
}
/// Sort a busfile on disk (i.e. without loading the entire thing into memory)
/// Works via `mergesort`:
/// 1. split the busfile into separate chunks on disk: Temporary directory is used
/// 2. sort the chunks (in memory) individually
/// 3. merge the chunks: iterate over all chunks in parallel via [bustools::merger]
/// and aggregate records that might have been split across chunks
///
/// # Parameters:
/// * `busfile`: file to be sorted
/// * `outfile`: file to be sorted into
/// * `chunksize`: number of busrecords per chunk (this is how much is loaded into mem at any point).
///    `chunksize=10_000_000` is roughly a 300MB chunk on disk
/// 
pub fn sort_on_disk(busfile: &str, outfile: &str, chunksize: usize) {
    let reader = BusReader::new(busfile);
    let header = reader.bus_header.clone();

    let mut chunkfiles = Vec::new();

    println!("Sorting chunks");
    let tmpdir = tempdir().unwrap();

    for (i, record_chunk) in (&reader.chunks(chunksize)).into_iter().enumerate() {
        println!("Sorting {}th chunks", i);

        // sort the chunk in memory
        let in_mem_sort = sort_into_btree(record_chunk);

        //write current sorted file to disk
        let file_path = tmpdir.path().join(format!("tmp_{}.bus", i));
        let tmpfilename = file_path.to_str().unwrap().to_string();

        let mut tmpwriter = BusWriter::new(&tmpfilename, header.clone());

        for (_cbumi, record) in in_mem_sort {
            tmpwriter.write_record(&record);
        }
        chunkfiles.push(tmpfilename);
    }

    // merge all chunks
    println!("Merging {} chunks", chunkfiles.len());
    let mut writer = BusWriter::new(outfile, header);

    // gather the individual iterators for each chunk
    let mut iterator_map = HashMap::new();
    for file in chunkfiles.iter() {
        let iter = BusReader::new(file).groupby_cbumi();
        iterator_map.insert(file.to_string(), iter);
    }

    // each file itself is sorted
    // now we only have to merge them
    // if a single cb/umi is split over multiple records, this will put them back together
    // however, we need to aggregate their counts and sort them by EC
    let mi = MultiIterator::new(iterator_map);
    for (_cbumi, record_dict) in mi {
        let merged_records = merge_chunks(record_dict);  //takes care of aggregating across chunks and sorting
        writer.write_records(&merged_records);
    }
    //tmpfiles get clean up once tmpdir is dropped!
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use super::{sort_in_memory, sort_on_disk};
    use bustools::{
        io::{setup_busfile, BusHeader, BusReader, BusRecord, BusWriter},
        iterators::CbUmiGroupIterator,
    };

    #[test]
    fn test_merge_sorted_aggregated(){
        let input: HashMap<String, Vec<BusRecord>> = HashMap::from(
            [
                ("s1".to_string(), vec![
                    BusRecord {CB:0 , UMI: 1, EC:0, COUNT:1 , FLAG:0},
                    BusRecord {CB:0 , UMI: 0, EC:1, COUNT:1 , FLAG:0},
                ]),
                ("s2".to_string(), vec![
                    BusRecord {CB:0 , UMI: 0, EC:0, COUNT:1 , FLAG:0},
                    BusRecord {CB:0 , UMI: 1, EC:0, COUNT:1 , FLAG:0},
                ]),                
            ]);
        let merged_records = super::merge_chunks(input);

        assert_eq!(merged_records, vec![
            BusRecord {CB:0 , UMI: 0, EC:0, COUNT:1 , FLAG:0},
            BusRecord {CB:0 , UMI: 0, EC:1, COUNT:1 , FLAG:0},
            BusRecord {CB:0 , UMI: 1, EC:0, COUNT:2 , FLAG:0}
        ])
    }

    #[test]
    fn test_sort_in_memory() {
        // this is the correct order here:
        let r1 = BusRecord { CB: 0, UMI: 1, EC: 0, COUNT: 12, FLAG: 0 };
        let r2 = BusRecord { CB: 0, UMI: 1, EC: 1, COUNT: 2, FLAG: 0 };
        let r3 = BusRecord { CB: 0, UMI: 2, EC: 0, COUNT: 12, FLAG: 0 };
        let r4 = BusRecord { CB: 1, UMI: 1, EC: 1, COUNT: 2, FLAG: 0 };
        let r5 = BusRecord { CB: 1, UMI: 2, EC: 1, COUNT: 2, FLAG: 0 };
        let r6 = BusRecord { CB: 2, UMI: 1, EC: 1, COUNT: 2, FLAG: 0 };

        let unsorted_records = vec![
            r6.clone(),
            r4.clone(),
            r1.clone(),
            r2.clone(),
            r5.clone(),
            r3.clone(),
        ];
        let (busname, _dir) = setup_busfile(&unsorted_records);

        let outpath = _dir.path().join("bustools_test_sorted.bus");
        let outfile = outpath.to_str().unwrap();

        sort_in_memory(&busname, outfile);

        let b = BusReader::new(outfile);
        let v: Vec<BusRecord> = b.collect();

        assert_eq!(v, vec![r1, r2, r3, r4, r5, r6]);
    }

    #[test]
    fn test_sort_on_disk() {
        // lets use chunksize 2 and split records over chunks on purpose

        let r1 = BusRecord { CB: 0, UMI: 1, EC: 0, COUNT: 12, FLAG: 0 };
        let r2 = BusRecord { CB: 0, UMI: 1, EC: 1, COUNT: 2, FLAG: 0 };
        let r3 = BusRecord { CB: 0, UMI: 2, EC: 0, COUNT: 12, FLAG: 0 };
        let r4 = BusRecord { CB: 1, UMI: 1, EC: 1, COUNT: 2, FLAG: 0 };
        let r5 = BusRecord { CB: 1, UMI: 2, EC: 1, COUNT: 2, FLAG: 0 };
        let r6 = BusRecord { CB: 2, UMI: 1, EC: 1, COUNT: 2, FLAG: 0 };
        let r7 = BusRecord { CB: 2, UMI: 1, EC: 0, COUNT: 2, FLAG: 0 };

        let unsorted_records = vec![
            // chunk 1
            r6.clone(),
            r4.clone(),
            // chunk 2
            r1.clone(),
            r7.clone(),
            // chunk 3
            r5.clone(),
            r3.clone(),
            // chunk 4
            r2.clone(),
        ];

        let (busname, _dir) = setup_busfile(&unsorted_records);
        let outpath = _dir.path().join("bustools_test_sorted.bus");
        let outfile = outpath.to_str().unwrap();

        sort_on_disk(&busname, outfile, 2);

        let b = BusReader::new(outfile);

        // the followug doesnt work: r1 and r2 are both (0,1) and hence their order is arbitray
        // assert_eq!(b.collect(), vec![r1, r2, r3, r4, r5, r6, r7]);

        // instead check the sorting of the file implicitely
        let n: usize = b.groupby_cbumi().map(|(_, rlist)| rlist.len()).sum();
        assert_eq!(n, 7)
    }

    use rand::distributions::{Distribution, Uniform};

    #[test]
    fn test_random_file_sort() {
        let cb_len = 16;
        let umi_len = 12;
        // let n_records = 10_000_000;
        // let chunksize = 1_000_000;

        let n_records = 10_000;
        let chunksize = 1_000;

        let cb_distr = Uniform::from(0..10000);
        let umi_distr = Uniform::from(0..10000);
        let mut rng = rand::thread_rng();

        use tempfile::tempdir;
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_bus_sort_random.bus");
        let outfile = file_path.to_str().unwrap();

        let mut writer = BusWriter::new(outfile, BusHeader::new(cb_len, umi_len, 20));
        for _ in 0..n_records {
            let cb = cb_distr.sample(&mut rng);
            let umi = umi_distr.sample(&mut rng);

            let r = BusRecord { CB: cb, UMI: umi, EC: 0, COUNT: 1, FLAG: 0 };
            writer.write_record(&r);
        }
        drop(writer); //make sure everything is written

        // sort it
        let sortec_path = dir.path().join("test_bus_sort_random_sorted.bus");
        let sorted_out = sortec_path.to_str().unwrap();
        sort_on_disk(&outfile, sorted_out, chunksize);

        // check if sorted
        let b = BusReader::new(sorted_out);
        let n: usize = b.groupby_cbumi().map(|(_, rlist)| rlist.len()).sum();
        assert_eq!(n, n_records)
    }

    mod sort_into_btree {
        use bustools::io::BusRecord;

        use crate::sort;
        #[test]
        fn test_simple(){
            let v = vec![
                BusRecord {CB: 1, UMI: 0, EC: 0, COUNT:1, FLAG: 0},
                BusRecord {CB: 0, UMI: 0, EC: 0, COUNT:1, FLAG: 0},
                BusRecord {CB: 0, UMI: 1, EC: 0, COUNT:1, FLAG: 0},
                ];
            let sorted_set = crate::sort::sort_into_btree(v.into_iter(), );
            assert_eq!(sorted_set.len(), 3);

            let umis: Vec<_> = sorted_set.iter().map(|(_,r)| r.UMI).collect();
            assert_eq!(umis, vec![0,1,0]);
        }
        #[test]
        fn test_ec_sorted(){
            let v = vec![
                BusRecord {CB: 0, UMI: 0, EC: 100, COUNT:1, FLAG: 0},
                BusRecord {CB: 0, UMI: 0, EC: 10, COUNT:1, FLAG: 0},
                BusRecord {CB: 0, UMI: 0, EC: 1, COUNT:1, FLAG: 0},
                ];
            let sorted_set = crate::sort::sort_into_btree(v.into_iter(), );
            assert_eq!(sorted_set.len(), 3);

            let ecs: Vec<_> = sorted_set.iter().map(|(_,r)| r.EC).collect();
            assert_eq!(ecs, vec![1,10,100]);
        }

        #[test]
        fn test_merge(){
            let v = vec![
                BusRecord {CB: 0, UMI: 0, EC: 0, COUNT:1, FLAG: 0},
                BusRecord {CB: 0, UMI: 0, EC: 0, COUNT:1, FLAG: 0},
                BusRecord {CB: 0, UMI: 0, EC: 0, COUNT:1, FLAG: 0},
                ];
            let sorted_set = crate::sort::sort_into_btree(v.into_iter(), );
            assert_eq!(sorted_set.len(), 1);

            let counts: Vec<_> = sorted_set.iter().map(|(_,r)| r.COUNT).collect();
            assert_eq!(counts, vec![3]);
        }        
    }
}
