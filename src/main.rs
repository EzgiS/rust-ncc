#![allow(clippy::too_many_arguments)]
//! The entry point.
// use rust_ncc::world::hardio::Format;
use rand::distributions::Uniform;
use rand::Rng;
use rust_ncc::{experiments, world, DEFAULT_OUTPUT_DIR};
use std::path::PathBuf;
use std::time::Instant;

pub struct Args {
    seed: Option<u64>,
    z: bool,
    coa: Option<f64>,
}

fn main() {
    let mut args_vec: Vec<Args> = vec![];
    for &seed in [None].iter() {
        for &z in [true, false].iter() {
            for &coa in [Some(24.0)].iter() {
                args_vec.push(Args {
                    seed,
                    z,
                    coa
                })
            }
        }
    }
    for args in args_vec {
        let Args {
            seed, z, coa
        } = args;
        println!("seed: {:?}, z: {}, coa: {:?}", seed, z, coa);
        let exp = experiments::pair::generate(z, coa, seed);

        let mut w = world::World::new(
            exp,
            Some(PathBuf::from(DEFAULT_OUTPUT_DIR)),
            10,
            100,
        );

        let now = Instant::now();
        w.simulate(3.0 * 3600.0, true);

        println!(
            "Simulation complete. {} s.",
            now.elapsed().as_secs()
        );
    }
}
