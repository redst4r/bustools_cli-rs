//! # bustools_cli
//!
//! The command line interface for my rust version of [bustools](https://github.com/BUStools/bustools).
//!
//! At this point, it's **far from complete and correct**, but rather a project to learn rust.
//! Most functionality comes from a companion crate, [bustools](https://crates.io/crates/bustools).
//!
//! # CLI
//! `bustools <command>`
//! * `correct`: Correct busfiles via a whitelist
//! * `sort`: Sort the busfile by CB/UMI/EC
//! * `count`: Create a count-matrix (CB vs gene)
//! * `inspect`: Basic stats about a busfile (#records, #CBs etc..)
//!
//! Check the CLI help for arguments.
//!
#[allow(warnings)]
use bustools::consistent_genes::{GeneId, Genename, EC};
use bustools::io::{BusFolder, BusReader};
use bustools::iterators::CellGroupIterator;
use bustools::utils::int_to_seq;
use clap::{self, Args, Parser, Subcommand};
use itertools::Itertools;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use bustools::busz::compress_busfile;
use bustools::busz::decompress_busfile;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Path to output file
    #[clap(short = 'o', long = "output")]
    output: String,

    #[clap(subcommand)]
    command: MyCommand,
}

#[allow(non_camel_case_types)]
#[derive(Subcommand)]
enum MyCommand {
    busmerge(BusMergeArgs),
    count(CountArgs),
    count2(CountArgs),
    resolve_ec(ResolveArgs),
    inspect(InspectArgs),
    sort(SortArgs),
    getcb(GetCBArgs),
    butterfly(ButterflyArgs),
    correct(CorrectArgs),
    compress(CompressArgs),
    decompress(DecompressArgs),
}

/// compress a busfile
#[derive(Args)]
struct CompressArgs {
    /// Input: sorted busfile
    #[clap(long = "input", short = 'i')]
    input: String,

    /// Number of rows to compress as a single block.
    #[clap(long = "chunk-size", short='N')]
    chunksize: usize,
}

/// Decompress a busfile
#[derive(Args)]
struct DecompressArgs {
    /// Input: compressed busfile
    #[clap(long = "input", short = 'i')]
    input: String,
}


/// correct CBs with whitelist
#[derive(Args)]
struct CorrectArgs {
    /// Input busfile
    #[clap(long = "ifile", short = 'i')]
    inbus: String,

    /// Cell Barcode Whitelist
    #[clap(long = "whitelist")]
    whitelist: String,
}

/// Buttefly/ amplification profile
#[derive(Args)]
struct ButterflyArgs {
    /// input busfolder
    #[clap(long = "ifile", short = 'i')]
    inbus: String,
    /// Transcript-to-gene file
    #[clap(long = "t2g")]
    t2g: String,
    /// CB-UMI entries with multiple ECs will be collapsed into a single record (if they are consistent with a single gene)
    #[clap(long = "collapse")]
    collapse_ec: bool,
}

/// Sort busfile by CB/UMI/EC
#[derive(Args)]
struct SortArgs {
    /// input busfolder
    #[clap(long = "ifile", short = 'i')]
    inbus: String,
}

/// count the mRNAs  per cell and write to file
#[derive(Args)]
struct GetCBArgs {
    /// input busfolder
    #[clap(long = "ifile", short = 'i')]
    inbus: String,
}

/// countmatrix from busfile
#[derive(Args)]
struct CountArgs {
    /// input busfolder
    #[clap(long = "ifolder")]
    inbus: String,

    /// Transcript-to-gene file
    #[clap(long = "t2g")]
    t2g: String,

    /// ignore multimapped busrecords (same CB/UMI but different EC)
    #[clap(long = "ignoremm")]
    ignoremm: bool,
}

/// find overlap between busfiles and write out overlapping molecules
#[derive(Args)]
struct BusMergeArgs {
    /// 1st Input busfile
    #[clap(long = "i1")]
    inbus1: String,
    /// 2nd Input busfile
    #[clap(long = "i2")]
    inbus2: String,

    /// 1st output busfile
    #[clap(long = "o1")]
    outbus1: String,
    /// 2nd output busfile
    #[clap(long = "o2")]
    outbus2: String,
}

/// resovle an EC into gene names
#[derive(Args)]
struct ResolveArgs {
    /// input busfolder
    #[clap(long = "ifolder")]
    inbus: String,
    #[clap(long = "t2g")]
    /// Transcript-to-gene file
    t2g: String,

    /// Equivalence class to query genes for
    #[clap(long = "ec")]
    ec: u32,
}

/// Inspect busfile for stats
#[derive(Args)]
struct InspectArgs {
    /// input busfolder
    #[clap(short = 'i', long = "input")]
    inbus: String,
}

use bustools_cli::busmerger;
use bustools_cli::butterfly;
use bustools_cli::correct;
use bustools_cli::count;
use bustools_cli::count2;
use bustools_cli::inspect;
use bustools_cli::sort;

fn main() {
    let cli = Cli::parse();
    match cli.command {
        MyCommand::busmerge(args) => {
            println!("Doing bus merging");
            busmerger::merge_busfiles_on_overlap(
                &args.inbus1,
                &args.inbus2,
                &args.outbus1,
                &args.outbus2,
            )
        }
        MyCommand::count(args) => {
            println!("Doing count");

            fs::create_dir(&cli.output).unwrap();

            let bfolder = BusFolder::new(&args.inbus, &args.t2g);
            let c = count::count(&bfolder, args.ignoremm);

            c.write(&cli.output);
        }
        MyCommand::count2(args) => {
            println!("Doing count");
            fs::create_dir(&cli.output).unwrap();

            let bfolder = BusFolder::new(&args.inbus, &args.t2g);

            let c = count2::count(&bfolder, args.ignoremm);
            c.write(&cli.output);
        }

        MyCommand::resolve_ec(args) => {
            println!("Doing resolve");
            let bfolder = BusFolder::new(&args.inbus, &args.t2g);
            let mut genes: Vec<&GeneId> = bfolder.ec2gene.get_genes(EC(args.ec)).iter().collect();
            genes.sort();
            println!("EC {} -> {:?}", args.ec, genes);

            let mut genenames: Vec<Genename> = bfolder
                .ec2gene
                .get_genenames(EC(args.ec))
                .into_iter()
                .collect();
            genenames.sort();

            println!("EC {} -> {:?}", args.ec, genenames);
        }
        MyCommand::inspect(args) => {
            inspect::inspect(&args.inbus);
        }

        MyCommand::getcb(args) => {
            let fh = File::create(cli.output).unwrap();
            let mut writer = BufWriter::new(fh);
            // let cb_len = 16;

            let reader = BusReader::new(&args.inbus);
            let params = reader.get_params();
            let cb_len = params.cb_len as usize;
            let bus_cb = reader
                .groupby_cb()
                .map(|(cb, records)| {
                    (
                        // CB,decoded
                        int_to_seq(cb, cb_len),
                        // number of UMIs
                        records.iter().map(|r| r.UMI).unique().count(),
                    )
                });

            for (cb, nrecords) in bus_cb {
                writeln!(writer, "{},{}", cb, nrecords).unwrap();
            }
        }
        MyCommand::sort(args) => {
            let chunksize = 10_000_000; // roughly 300MB on disk
            sort::sort_on_disk(&args.inbus, &cli.output, chunksize)
        }
        MyCommand::butterfly(args) => {
            let bfolder = BusFolder::new(&args.inbus, &args.t2g);
            let cuhist = butterfly::make_ecs(&bfolder, args.collapse_ec);
            cuhist.to_disk(&cli.output);
        }
        MyCommand::correct(args) => {
            correct::correct(&args.inbus, &cli.output, &args.whitelist);
        }
        MyCommand::compress(args) => {
            compress_busfile(&args.input, &cli.output, args.chunksize);
        },
        MyCommand::decompress(args) => {
            decompress_busfile(&args.input, &cli.output);
        },
    }
}
