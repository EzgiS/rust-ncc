#![allow(clippy::too_many_arguments)]
//! The entry point.
// use rust_ncc::world::hardio::Format;
use rand::distributions::Uniform;
use rand::Rng;
use rust_ncc::{experiments, world, DEFAULT_OUTPUT_DIR};
use std::path::PathBuf;
use std::time::Instant;

fn main() {
    let seed_list: Vec<u64> = vec![4, 44, 444, 4444];
    for seed in seed_list {
        let mut rng = rand::thread_rng();
        // let seed = 1111; //rng.sample(Uniform::new(0, 10000));
        println!("seed: {}", seed);
        let exp = experiments::pair::generate(Some(seed), true);

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
