use crate::cell::chemistry::{distrib_gens, RgtpDistribution};
use crate::exp_setup::{
    CellGroup, Experiment, ExperimentType, GroupBBox,
};
use crate::math::v2d::V2d;
use crate::parameters::quantity::{Length, Quantity};
use crate::parameters::{
    CharQuantities, RawCloseBounds, RawInteractionParams,
    RawParameters, RawPhysicalContactParams,
};
use crate::Directories;

use crate::exp_setup::defaults::{
    ADH_MAG, CHAR_QUANTS, PHYS_CLOSE_DIST,
    RAW_COA_PARAMS_WITH_ZERO_MAG, RAW_PARAMS, RAW_WORLD_PARAMS,
};
use crate::exp_setup::exp_parser::ExperimentArgs;
use crate::exp_setup::markers::mark_verts;
use crate::utils::pcg32::Pcg32;
use rand::SeedableRng;

/// Generate the group layout to use for this experiment.
fn group_bbox(
    group_ix: usize,
    char_quants: &CharQuantities,
    raw_params: &RawParameters,
) -> Result<GroupBBox, String> {
    // specify initial location of group centroid
    let bottom_left = V2d {
        x: char_quants
            .normalize(&raw_params.cell_diam.scale(group_ix as f64)),
        y: char_quants.normalize(&Length(0.0)),
    };
    let r = GroupBBox {
        width: 1,
        height: 1,
        bottom_left,
    };
    if r.width * r.height < 1 {
        Err(String::from(
            "Group layout area is too small to contain required number of cells.",
        ))
    } else {
        Ok(r)
    }
}

fn raw_params(group_ix: usize, randomization: bool) -> RawParameters {
    let right = mark_verts(&[0, 1, 2, 3]);
    let left = mark_verts(&[8, 9, 10, 11]);

    let (specific_rac, specific_rho) = match group_ix {
        0 => (right, left),
        1 => (left, right),
        _ => panic!("received group ix > 1"),
    };

    let rac_distrib =
        distrib_gens::specific_uniform(0.3, specific_rac);
    let init_rac = RgtpDistribution::new(rac_distrib, rac_distrib);

    let rho_distrib =
        distrib_gens::specific_uniform(0.3, specific_rho);
    let init_rho = RgtpDistribution::new(rho_distrib, rho_distrib);

    RAW_PARAMS
        .modify_randomization(randomization)
        .modify_init_rac(init_rac)
        .modify_init_rho(init_rho)
}

#[allow(clippy::too_many_arguments)]
fn make_cell_group(
    group_ix: usize,
    char_quants: &CharQuantities,
    randomization: bool,
    num_cells: usize,
) -> CellGroup {
    let raw_params = raw_params(group_ix, randomization);
    let parameters = raw_params.refine(char_quants);
    CellGroup {
        num_cells,
        layout: group_bbox(group_ix, char_quants, &raw_params)
            .unwrap(),
        parameters,
    }
}

/// Define the cell groups that will exist in this experiment.
fn make_cell_groups(
    char_quants: &CharQuantities,
    randomization: bool,
    num_cells: usize,
) -> Vec<CellGroup> {
    (0..num_cells)
        .map(|group_ix| {
            make_cell_group(group_ix, char_quants, randomization, 1)
        })
        .collect::<Vec<CellGroup>>()
}

pub fn generate(
    dirs: Directories,
    args: ExperimentArgs,
) -> Vec<Experiment> {
    let ExperimentArgs {
        ty,
        final_t,
        cil_mag,
        coa_mag,
        adh_mag: adh_scale,
        cal_mag,
        randomization,
        seeds,
        file_name: toml_name,
        snap_period,
        max_on_ram,
        int_opts,
        ..
    } = args;

    let (num_cells, py_main) =
        if let ExperimentType::PyCompare { num_cells, py_main } = &ty
        {
            (
                *num_cells,
                py_main.as_ref().unwrap_or(&dirs.py_main).clone(),
            )
        } else {
            panic!(format!(
            "expected ExperimentType::PyCompare, instead found: {:?}",
            ty
        ));
        };

    seeds
        .iter()
        .map(|&seed| {
            let rng = Pcg32::seed_from_u64(seed);

            let char_quants = *CHAR_QUANTS;
            let raw_world_params = RAW_WORLD_PARAMS
                .modify_interactions(RawInteractionParams {
                    coa: coa_mag.map(|mag| {
                        RAW_COA_PARAMS_WITH_ZERO_MAG.modify_mag(mag)
                    }),
                    chem_attr: None,
                    bdry: None,
                    phys_contact: RawPhysicalContactParams {
                        range: RawCloseBounds {
                            zero_at: PHYS_CLOSE_DIST.scale(3.0),
                            one_at: *PHYS_CLOSE_DIST,
                        },
                        adh_mag: adh_scale.map(|x| ADH_MAG.scale(x)),
                        cal_mag,
                        cil_mag,
                    },
                });
            let world_params = raw_world_params.refine(&char_quants);
            let cgs = make_cell_groups(
                &char_quants,
                randomization,
                num_cells,
            );

            Experiment {
                ty: (&ty).clone(),
                name: format!("{}_seed={}", toml_name, seed,),
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
                py_main: Some(py_main.clone()),
            }
        })
        .collect()
}