#![allow(clippy::too_many_arguments)]
//! The entry point.
#[cfg(feature = "animate")]
mod animator;
mod cell;
mod experiments;
mod interactions;
mod math;
mod parameters;
mod utils;
mod world;

#[cfg(feature = "animate")]
use crate::animator::create_animation;
use crate::world::hardio::Format;
use rand::distributions::Uniform;
use rand::Rng;
use std::path::PathBuf;
use std::time::Instant;

// /// Number of vertices per model cell.
pub const NVERTS: usize = 16;

fn main() {
    let mut rng = rand::thread_rng();
    let seed = rng.sample(Uniform::new(0, 10000));
    println!("seed: {}", seed);
    let exp = experiments::separated_pair::generate(Some(seed));

    let output_dir = PathBuf::from("./output");
    let mut w = world::World::new(exp, output_dir.clone());

    let now = Instant::now();
    // to run longer change final_tpoint to 6 etc.
    w.simulate(6.0 * 3600.0, 30);
    println!("Simulation complete. {} s.", now.elapsed().as_secs());
    let now = Instant::now();
    w.save_history(true, vec![Format::Cbor, Format::Bincode])
        .unwrap();
    println!(
        "Finished saving history. {} s.",
        now.elapsed().as_secs()
    );

    #[cfg(feature = "animate")]
    create_animation(&w.history, &output_dir.join("out.mp4"));
}
