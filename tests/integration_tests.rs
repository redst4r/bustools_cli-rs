use std::{fs, time::Instant};
use bustools_cli::{count::count, count2, correct::correct, butterfly::make_ecs};
use bustools::io::{BusFolder, BusReader, write_partial_busfile};
use bustools::iterators::CellGroupIterator;
use bustools_cli::countmatrix::CountMatrix;

// pub const TEST_T2G: &str = "/home/michi/transcripts_to_genes.txt";
// pub const TEST_BUSFILE: &str = "/home/michi/mounts/TB4drive/ISB_data/LT_pilot/LT_pilot/kallisto_quant/DSP1/kallisto/sort_bus/bus_output/output.corrected.sort.bus";
// pub const TEST_BUSFOLDER: &str = "/home/michi/mounts/TB4drive/ISB_data/LT_pilot/LT_pilot/kallisto_quant/DSP1/kallisto/sort_bus/bus_output/";

// pub const TEST_T2G: &str = "/home/michi/mounts/TB4drive/kallisto_resources/transcripts_to_genes.txt";
// pub const TEST_BUSFILE: &str = "/home/michi/ISB_data/LT_pilot/LT_pilot/kallisto_quant/DSP1/kallisto/sort_bus/bus_output/output.corrected.sort.bus";
// pub const TEST_BUSFOLDER: &str = "/home/michi/mounts/TB4drive/ISB_data/LT_pilot/LT_pilot/kallisto_quant/DSP1/kallisto/sort_bus/bus_output/";


pub const TEST_T2G: &str = "/home/michi/bus_testing/transcripts_to_genes.txt";
pub const TEST_BUSFILE: &str = "/home/michi/bus_testing/bus_output/output.corrected.sort.bus";
pub const TEST_BUSFOLDER: &str = "/home/michi/bus_testing/bus_output/";
pub const TEST_WHITELIST: &str = "/home/michi/bus_testing/3M-february-2018.txt";


#[test]
fn test_count_vs_bustools() {
    // comparing our implementation vs kallisto-bustools on a real bus file
    // run:
    // RUST_BACKTRACE=1 cargo test --release --package rustbustools --lib -- --nocapture count::test::test_vs_bustools
    use std::process::Command;

    let outfolder = "/tmp/bustest_rust";
    let outfolder_kallisto = "/tmp/bustest_kallisto";


    let bfolder = BusFolder::new(TEST_BUSFOLDER, TEST_T2G);
    let tfile = bfolder.get_transcript_file();
    let ecfile = bfolder.get_ecmatrix_file();
    let bfile = bfolder.get_busfile();

    fs::create_dir_all(outfolder).expect("Failed to create outfolder");
    fs::create_dir_all(outfolder_kallisto).expect("Failed to create outfolder_kallisto");

    // -------------------
    // Doing our own count
    // -------------------
    println!("Doing count::count");
    let now = Instant::now();
    let c = count(&bfolder, false);
    let elapsed_time = now.elapsed();
    println!("count::count in in {:?}", elapsed_time);
    c.write(outfolder);

    println!("Doing count::count2");
    let now = Instant::now();
    let c2 = count2::count(&bfolder, false);
    let elapsed_time = now.elapsed();
    println!("count2::count in in {:?}", elapsed_time);
    assert_eq!(c2, c);

    // -------------------
    // Bustools count
    // -------------------
    // Command

    let bustools_binary = "/home/michi/miniconda3_newtry/envs/nextflow_bioinformatics/bin/bustools";
    println!("Doing kallisto::count");
    let now = Instant::now();
    let output = Command::new(bustools_binary)
        .arg("count")
        .arg("-o")
        .arg(format!("{outfolder_kallisto}/gene"))
        .arg("-e")
        .arg(ecfile)
        .arg("-g")
        .arg(TEST_T2G)
        .arg("-t")
        .arg(tfile)
        .arg("--genecounts")
        .arg(bfile)
        .output().unwrap();
    let elapsed_time = now.elapsed();
    println!("kallisto::count in in {:?}", elapsed_time);

    println!("status: {}", output.status);

    // -------------------
    // Comparing results
    // -------------------
    let cmat_kallisto = CountMatrix::from_disk(
        &format!("{outfolder_kallisto}/gene.mtx"),
        &format!("{outfolder_kallisto}/gene.barcodes.txt"),
        &format!("{outfolder_kallisto}/gene.genes.txt"),
    );

    let cmat_rust = c;

    let sum1: i32 = cmat_kallisto.matrix.iter().map(|(v, _s)| *v).sum();
    let sum2: i32 = cmat_rust.matrix.iter().map(|(v, _s)| *v).sum();
    assert_eq!(sum1, sum2);
    assert_eq!(cmat_kallisto, cmat_rust);
}

// #[test]
fn test_cb_iter_speed() {
    let n = 100000;

    let b = BusReader::new(TEST_BUSFOLDER);
    let biter2 = b.groupby_cb();

    let now = Instant::now();
    let _s2: Vec<_> = biter2.take(n).map(|(_a, records)| records).collect();
    let elapsed_time = now.elapsed();
    println!(
        "Running CellIterator({}) took {} seconds.",
        n,
        elapsed_time.as_secs()
    );
}

// #[test]
fn test_write() {
    // let outname = "/home/michi/bus_testing/bus_output_short/output.corrected.sort.bus";
    // write_partial_busfile(TEST_BUSFILE, outname, 10_000_000);

    let outname = "/home/michi/bus_testing/bus_output_shorter/output.corrected.sort.bus";
    write_partial_busfile(TEST_BUSFILE, outname, 1_500_000);
}

#[test]
fn test_count() {
    let b = BusFolder::new(TEST_BUSFOLDER, TEST_T2G);
    let count_matrix: CountMatrix = count(&b, false);
    count_matrix.write("/tmp");
    // count_bayesian(b)
}

#[test]
fn test_correct_real_file() {
    correct(TEST_BUSFILE, "/tmp/corrected.bus", TEST_WHITELIST)
}

// #[test]
// fn test_correct_bk() {
//     let the_whitelist = "/home/michi/bus_testing/3M-february-2018.txt";
//     let wl = load_whitelist(the_whitelist);
//     let mut bk: BkTree<String> = BkTree::new(my_hamming);
//     bk.insert_all(wl.into_iter());

//     let r = bk.find("TAACCTGAGACTCGGA".to_string(), 1);
//     println!("{:?}", r);
// }

#[test]
pub fn test_butterfly() {
    let b = BusFolder::new(
        TEST_BUSFOLDER,
        TEST_T2G,
    );
    let h = make_ecs(&b, true);
    println!("{:?}", h);
}