//! concatenate busfiles
//! 

use std::collections::HashMap;

use bustools::{io::{BusReader, BusWriter}, iterators::CbUmiGroupIterator, merger::MultiIterator};

use crate::sort::merge_chunks;


///
// fn concat_internal(x: HashMap<String, impl CUGIterator>) -> impl Iterator<Item=BusRecord> {

//     let itermap: HashMap<String,_> = x.into_iter().map(|(s, iter)| (s, iter.groupby_cbumi())).collect();

//     let it = MultiIterator::new(itermap)
//         .map(|(_cbumi, rdict)|
//             merge_chunks(rdict)
//         )
//         .flatten();
//     it
// }

/// Concatenate several busfiles
/// 
/// Assumes that each file is sorted
/// If a record (CB/UMI/EC) is found in more than one busfile, its count is aggregated
/// (also if the same CB/UMI/EC is present in the same file)
pub fn concat_bus(filenames: Vec<String>, outfile: &str) {

    let mut readers = HashMap::new();
    for f in filenames.iter() {
        readers.insert(
            f.to_owned(),
            BusReader::new(f)
        );
    }

    let params = readers[&filenames[0]].get_params().clone();
    
    // assert all busfiles have ethe same parameters
    for (_, r) in readers.iter() {
        let pa = r.get_params().clone();
        assert_eq!(pa, params, "missmatched Header parameters in busfiles")
    }

    // merge all chunks
    println!("Merging {} chunks", filenames.len());
    let mut writer = BusWriter::new(outfile, params);

    let iterator_map: HashMap<String, _> = readers
        .into_iter()
        .map(|(f, read)| 
            (f.to_owned(), read.groupby_cbumi())
         ).collect();
    
    // each file itself is sorted
    // now we only have to merge them
    // if a single cb/umi is split over multiple records, this will put them back together
    // however, we need to aggregate their counts and sort them by EC

    let it = MultiIterator::new(iterator_map)
        .flat_map(|(_cbumi, rdict)|
            merge_chunks(rdict)
        );
    writer.write_iterator(it);
}

#[cfg(test)]
mod test {
    use bustools::io::{setup_busfile, BusReader, BusRecord};

    use super::concat_bus;

    #[test]
    fn test_concat(){
        let r1 = BusRecord { CB: 0, UMI: 1, EC: 0, COUNT: 12, FLAG: 0 };
        let r2 = BusRecord { CB: 0, UMI: 1, EC: 1, COUNT: 2, FLAG: 0 };
        let r3 = BusRecord { CB: 0, UMI: 1, EC: 1, COUNT: 1, FLAG: 0 };  // should be aggr with prev

        let r4 = BusRecord { CB: 1, UMI: 0, EC: 0, COUNT: 1, FLAG: 0 };  // shoudl aggr with s1

        let r5 = BusRecord { CB: 2, UMI: 0, EC: 0, COUNT: 1, FLAG: 0 };  // shoudl NOT aggr with s2, different EC 


        let s1 = BusRecord { CB: 1, UMI: 0, EC: 0, COUNT: 2, FLAG: 0 };
        let s2 = BusRecord { CB: 2, UMI: 0, EC: 1, COUNT: 2, FLAG: 0 };


        let (busname1, _dir1) = setup_busfile(&vec![r1.clone() ,r2.clone() ,r3.clone() ,r4.clone() , r5.clone()]);
        let (busname2, _dir2) = setup_busfile(&vec![s1.clone(), s2.clone()]);

        concat_bus(vec![busname1, busname2], "/tmp/concat.bus");

        let reader = BusReader::new("/tmp/concat.bus");

        let exp = vec![
            r1,
            BusRecord { CB: 0, UMI: 1, EC: 1, COUNT: 3, FLAG: 0 }, 
            BusRecord { CB: 1, UMI: 0, EC: 0, COUNT: 3, FLAG: 0 },
            r5,
            s2
        ];

        assert_eq!(exp , reader.collect::<Vec<_>>());

    }
}