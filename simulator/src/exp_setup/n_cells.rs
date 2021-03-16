use crate::cell::chemistry::{distrib_gens, RgtpDistribution};
use crate::exp_setup::defaults::RAW_COA_PARAMS_WITH_ZERO_MAG;
use crate::exp_setup::exp_parser::ExperimentArgs;
use crate::exp_setup::{defaults, CellGroup, Experiment, ExperimentType, GroupBBox};
// use crate::mark_verts;
use crate::math::v2d::V2d;
use crate::parameters::quantity::{Length, Quantity};
use crate::parameters::{
    CharQuantities, RawCloseBounds, RawInteractionParams, RawParameters, RawPhysicalContactParams,
};
use crate::utils::pcg32::Pcg32;
use crate::Directories;
use rand::SeedableRng;

/// Generate the group bounding box to use for this experiment.
fn group_bbox(num_cells: usize, char_quants: &CharQuantities) -> Result<GroupBBox, String> {
    // specify initial location of group centroid
    let centroid = V2d {
        x: char_quants.normalize(&Length(0.0)),
        y: char_quants.normalize(&Length(0.0)),
    };
    let side_len = (num_cells as f64).sqrt();
    let r = GroupBBox {
        width: side_len.ceil() as usize,
        height: (num_cells as f64 / side_len).ceil() as usize,
        bottom_left: centroid,
    };
    if r.width * r.height < num_cells {
        Err(String::from(
            "Group layout area is too small to contain required number of cells.",
        ))
    } else {
        Ok(r)
    }
}

fn raw_params(rng: &mut Pcg32, randomization: bool) -> RawParameters {
    let init_rac = RgtpDistribution::new(
        distrib_gens::random(rng, 0.1),
        distrib_gens::random(rng, 0.1),
    );

    let init_rho = RgtpDistribution::new(
        distrib_gens::random(rng, 0.1),
        distrib_gens::random(rng, 0.1),
    );

    defaults::RAW_PARAMS
        .modify_randomization(randomization)
        .modify_init_rac(init_rac)
        .modify_init_rho(init_rho)
}

/// Define the cell groups that will exist in this experiment.
fn make_cell_groups(
    rng: &mut Pcg32,
    char_quants: &CharQuantities,
    num_cells: usize,
    randomization: bool,
) -> Vec<CellGroup> {
    vec![CellGroup {
        num_cells,
        layout: group_bbox(num_cells, char_quants).unwrap(),
        parameters: raw_params(rng, randomization).refine(char_quants),
    }]
}

pub fn generate(dirs: Directories, args: ExperimentArgs) -> Vec<Experiment> {
    let ExperimentArgs {
        toml_name,
        ty,
        final_t,
        cil_mag,
        coa_mag,
        cal_mag,
        adh_scale,
        snap_period,
        max_on_ram,
        seeds,
        int_opts,
    } = args;

    let num_cells = if let ExperimentType::NCells { num_cells } = &ty {
        *num_cells
    } else {
        panic!("Expected an n_cell experiment, but got: {:?}", ty)
    };
    println!("cil_mag: {:?}", cil_mag);
    println!("coa_mag: {:?}", coa_mag);
    println!("cal_mag: {:?}", cal_mag);
    println!("adh_scale: {:?}", adh_scale);

    seeds
        .iter()
        .map(|&seed| {
            let (mut rng, randomization) = match seed {
                Some(s) => (Pcg32::seed_from_u64(s), true),
                None => (Pcg32::from_entropy(), false),
            };

            let char_quants = *defaults::CHAR_QUANTS;
            let raw_world_params =
                defaults::RAW_WORLD_PARAMS.modify_interactions(RawInteractionParams {
                    coa: coa_mag.map(|mag| RAW_COA_PARAMS_WITH_ZERO_MAG.modify_mag(mag)),
                    chem_attr: None,
                    bdry: None,
                    phys_contact: RawPhysicalContactParams {
                        range: RawCloseBounds {
                            zero_at: defaults::PHYS_CLOSE_DIST.scale(2.0),
                            one_at: *defaults::PHYS_CLOSE_DIST,
                        },
                        adh_mag: adh_scale.map(|x| defaults::ADH_MAG.scale(x)),
                        cal_mag,
                        cil_mag,
                    },
                });
            println!(
                "raw_world_params.interactions.cil: {:?}",
                raw_world_params.interactions.phys_contact.cil_mag
            );
            println!(
                "raw_world_params.interactions.coa: {:?}",
                raw_world_params.interactions.coa
            );
            println!(
                "raw_world_params.interactions.cal: {:?}",
                raw_world_params.interactions.phys_contact.cal_mag
            );
            println!(
                "raw_world_params.interactions.adh: {:?}",
                raw_world_params.interactions.phys_contact.adh_mag
            );
            let world_params = raw_world_params.refine(&char_quants);
            let cgs = make_cell_groups(&mut rng, &char_quants, num_cells, randomization);

            let seed_string = seed.map_or(String::from("None"), |s| format!("{}", s));

            Experiment {
                ty: ty.clone(),
                name: format!("{}_seed={}", toml_name, seed_string),
                final_t,
                char_quants,
                world_params,
                cell_groups: cgs,
                rng,
                seed,
                snap_period,
                max_on_ram,
                int_opts,
                out_dir: (&dirs.out).clone(),
                py_main: None,
            }
        })
        .collect()
}
