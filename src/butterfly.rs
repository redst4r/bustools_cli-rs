//! `butterfly` provides quanification of amplification for a kallisto-bus scRNAseq experiment
//!
//! # Introduction
//! In scRNAseq, each mRNA is tagged uniquely (up to random collision) with CB+UMI.
//! Those are then amplified and sequenced.
//! If we see the same CB+UMI in multiple reads, we conclude that they are copies of the same original mRNA
//! For each unique mRNA we quantify its amplification factor and record the absolute
//! frequency of the ampification (i.e. how often do we see a 5x amplification,
//! 5reads for the same CB+UMI)
//!
//! This module quantifies the amplifcation (very fast!).
//! Further processing (where speed is not essential) is typically done in python,
//! e.g. saturation curves, unseen species estimators.
//!
//!
//! # Unseen species
//! Considering the CB+UMI as `species` and the reads as `observations`, this relates to the `unseen species` problem
//! How many unobserved `species` (CB+UMI) are there in the library given the amplification profile we've seen so far
//! While the module doesn't provide an unseen species estimator, it can easily be build on the [CUHistogram]
//!
//! # References
//! The whole concept is described (amongst other things) in this
//! [paper](https://genomebiology.biomedcentral.com/articles/10.1186/s13059-021-02386-z)
//!
//! # Examples
//! ```rust, no_run
//! # use bustools::io::BusFolder;
//! # use bustools::butterfly::make_ecs;
//! let b = BusFolder::new(
//!     "/path/to/bus/output",
//!     "/path/to/transcripts_to_genes.txt",
//! );
//! let h = make_ecs(&b, true);
//! // save the resulting frequency of frequency histogram to disk
//! // can be read in python for further processing (e.g. plot the saturation curves)
//! h.to_disk("/tmp/CU.csv")
//! ```

#![deny(missing_docs)]
use bustools::{
    consistent_genes::{find_consistent, MappingResult, Ec2GeneMapper, MappingMode, InconsistentResolution},
    io::BusFolder,
    iterators::CbUmiGroupIterator,
};
use core::panic;
use std::{collections::HashMap, fs::File, io::Write};

/// The basic unit of this module, a frequency of frequency histogram
///
/// Records how many copies (reads) per mRNA (CB-UMI) we see in a busfile.
/// Should be constructed with [make_ecs]
#[derive(Debug)]
pub struct CUHistogram {
    // amplification (nReads for a single molecule) vs frequency
    histogram: HashMap<usize, usize>,
}
impl CUHistogram {
    // todo: useless!
    // fn new(h: HashMap<usize, usize>) -> Self {
    //     CUHistogram { histogram: h }
    // }

    /// return the number of reads (#molecules * number of copies) in the busfile
    pub fn get_nreads(&self) -> usize {
        self.histogram
            .iter()
            .map(|(nreads, freq)| nreads * freq)
            .sum()
    }

    /// return the number of molecules (distince CB/UMI pairs) in the busfile
    pub fn get_numis(&self) -> usize {
        self.histogram.values().sum()
    }

    /// calcualte the fraction of single-copy molecules (FSCM) in the busfile
    pub fn get_fscm(&self) -> f64 {
        let n1 = *self.histogram.get(&1).unwrap_or(&0);
        (n1 as f64) / (self.get_numis() as f64)
    }

    /// write the CU histogram into a csv on disk
    pub fn to_disk(&self, fname: &str) {
        let mut fh = File::create(fname).unwrap();

        fh.write_all("Amplification,Frequency\n".as_bytes())
            .unwrap();

        for (n_reads, n_umis) in self.histogram.iter() {
            fh.write_all(format!("{},{}\n", n_reads, n_umis).as_bytes())
                .unwrap();
        }
    }
}

impl From<CUHistogram> for HashMap<usize, usize> {
    fn from(value: CUHistogram) -> Self {
        value.histogram
    }
}

/// Main function of this module: Quantities the amplification in the given busfolder
/// # Arguments
/// * `busfolder`: The folder containing the busfile, matric.ec etc...
/// * `collapse_ec`: How to handle identical CB-UMI but different EC:
///     - false: just ignore and lump the reads together irresepctive of EC
///     - true: check if they ECs are consistent (if yes, count as aggregate), if no, discard
pub fn make_ecs(busfolder: &BusFolder, mapping_mode: MappingMode) -> CUHistogram {
    let mut h: HashMap<usize, usize> = HashMap::new();

    let mut multimapped = 0;
    let mut inconsistent = 0;
    let mut total = 0;

    for ((_cb, _umi), recordlist) in busfolder.get_iterator().groupby_cbumi() {
        total += 1;
        match &mapping_mode {

            // check if we can uniquely match those read to the same gene
            // if not its either multimapped or inconsistent (could be a CB/UMI collision)            
            MappingMode::Gene(ecmapper, resolution_mode) => {
                match find_consistent(&recordlist, ecmapper) {
                    MappingResult::SingleGene(_) => {
                        // increment our histogram
                        let nreads: usize = recordlist.iter().map(|x| x.COUNT as usize).sum();
                        let freq = h.entry(nreads).or_insert(0);
                        *freq += 1;
                    }
                    MappingResult::Multimapped(_) => multimapped += 1,
                    // inconsistent, i.e mapping to two distinct genes
                    // the reasonable thin
                    MappingResult::Inconsistent => {
                        match resolution_mode {
                            InconsistentResolution::IgnoreInconsistent => {inconsistent += 1},
                            InconsistentResolution::AsDistinct => panic!("not implemented"),
                            InconsistentResolution::AsSingle => {
                                let nreads: usize = recordlist.iter().map(|x| x.COUNT as usize).sum();
                                let freq = h.entry(nreads).or_insert(0);
                                *freq += 1;
                            },
                        }
                    },
                }
            },
            MappingMode::EC(mapping_mode) => {
                // one could get cb/umi with multiple ECs
                match mapping_mode{

                    // just check if its a single bus record (multiple records would indicate multiple ECs)
                    InconsistentResolution::IgnoreInconsistent => {
                        if recordlist.len() == 1 {
                            let nreads = recordlist[0].COUNT as usize;
                            let freq = h.entry(nreads).or_insert(0);
                            *freq += 1;
                        } else {
                            inconsistent += 1
                        }
                    },
                    InconsistentResolution::AsDistinct => panic!("not implemented"),
                    InconsistentResolution::AsSingle => {
                        let nreads: usize = recordlist.iter().map(|x| x.COUNT as usize).sum();
                        let freq = h.entry(nreads).or_insert(0);
                        *freq += 1;
                    },
                }
            }
            // MappingMode::IgnoreMultipleCbUmi => todo!(),
        }
    }

    println!(
        "Total CB-UMI {}, Multimapped {} ({}%), Discarded/Inconsistent {} ({}%)",
        total,
        multimapped,
        (multimapped as f32) / (total as f32),
        inconsistent,
        (inconsistent as f32) / (total as f32)
    );
    CUHistogram { histogram: h }
}

#[cfg(test)]
mod testing {
    use crate::butterfly::{make_ecs, CUHistogram};
    use bustools::{
        consistent_genes::{Ec2GeneMapper, Genename, EC, MappingMode, InconsistentResolution},
        io::{BusFolder, BusRecord},
        utils::vec2set,
    };

    use statrs::assert_almost_eq;
    use std::collections::{HashMap, HashSet};

    #[test]
    pub fn testing() {
        let h: HashMap<usize, usize> = vec![(1, 2), (3, 3)].into_iter().collect();
        let c = CUHistogram { histogram: h };

        assert_eq!(c.get_nreads(), 11);
        assert_eq!(c.get_numis(), 5);
        assert_almost_eq!(c.get_fscm(), 2.0 / 5.0, 0.00000000000000001);
    }

    #[test]
    fn test_butterfly() {
        // create some fake EC-> Gene mapping
        let ec0 = vec2set(vec![Genename("A".to_string())]);
        let ec1 = vec2set(vec![Genename("B".to_string())]);
        let ec2 = vec2set(vec![Genename("A".to_string()), Genename("B".to_string())]);
        let ec3 = vec2set(vec![Genename("C".to_string()), Genename("D".to_string())]);

        let ec_dict: HashMap<EC, HashSet<Genename>> = HashMap::from([
            (EC(0), ec0.clone()),
            (EC(1), ec1.clone()),
            (EC(2), ec2.clone()),
            (EC(3), ec3.clone()),
        ]);
        let es = Ec2GeneMapper::new(ec_dict);

        // two inconsitent records, should be ignored?!
        let r1 = BusRecord { CB: 0, UMI: 1, EC: 0, COUNT: 12, FLAG: 0 };
        let r2 = BusRecord { CB: 0, UMI: 1, EC: 1, COUNT: 2, FLAG: 0 };
        // several single records
        let r3 = BusRecord { CB: 0, UMI: 2, EC: 0, COUNT: 12, FLAG: 0 };
        let r4 = BusRecord { CB: 1, UMI: 1, EC: 1, COUNT: 2, FLAG: 0 };
        let r5 = BusRecord { CB: 1, UMI: 2, EC: 1, COUNT: 2, FLAG: 0 };

        // two consitent records, diffrnt EC though
        // should count as a a single 4x record
        let r6 = BusRecord { CB: 2, UMI: 1, EC: 0, COUNT: 2, FLAG: 0 };
        let r7 = BusRecord { CB: 2, UMI: 1, EC: 2, COUNT: 2, FLAG: 0 };

        let records = vec![
            r1.clone(),
            r2.clone(),
            r3.clone(),
            r4.clone(),
            r5.clone(),
            r6.clone(),
            r7.clone(),
        ];

        let (_busname, _dir) = bustools::io::setup_busfile(&records);
        let b = BusFolder {
            foldername: _dir.path().to_str().unwrap().to_owned(),
        };

        // collapsing ECS, ignoreing inconsistents
        let mapping_mode = MappingMode::Gene(es.clone(), InconsistentResolution::IgnoreInconsistent);
        let h = make_ecs(&b, mapping_mode);
        let expected: HashMap<usize, usize> = vec![(12, 1), (2, 2), (4, 1)].into_iter().collect();
        assert_eq!(h.histogram, expected);

        // collapsing ECS, counting inconsistens as a single molecule
        let mapping_mode = MappingMode::Gene(es, InconsistentResolution::AsSingle);
        let h = make_ecs(&b, mapping_mode);
        let expected: HashMap<usize, usize> = vec![(12, 1), (2, 2), (4, 1), (14,1)].into_iter().collect();
        assert_eq!(h.histogram, expected);



        // not collapsing ECs
        let mapping_mode = MappingMode::EC(InconsistentResolution::IgnoreInconsistent);
        let h = make_ecs(&b, mapping_mode);
        let expected: HashMap<usize, usize> = vec![
            (12, 1),
            (2, 2),
        ]
        .into_iter()
        .collect();

        assert_eq!(h.histogram, expected);
    }
}
