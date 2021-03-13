#![allow(unused)]
use crate::cell::chemistry::{
    DistributionScheme, DistributionType, RgtpDistribution,
};
use crate::experiments::{
    gen_default_adhesion_mag, gen_default_char_quants,
    gen_default_phys_contact_dist, gen_default_raw_params,
    gen_default_vertex_viscosity, CellGroup, Experiment, GroupBBox,
};
use crate::interactions::dat_sym2d::SymCcDat;
use crate::math::v2d::V2D;
use crate::parameters::quantity::{Force, Length, Quantity};
use crate::parameters::{
    CharQuantities, CoaParams, PhysicalContactParams, RawCloseBounds,
    RawCoaParams, RawInteractionParams, RawParameters,
    RawPhysicalContactParams, RawWorldParameters,
};
use crate::utils::pcg32::Pcg32;
use crate::NVERTS;
use rand::SeedableRng;

/// Generate the group layout to use for this experiment.
fn group_layout(
    num_cells: usize,
    char_quants: &CharQuantities,
) -> Result<GroupBBox, String> {
    // specify initial location of group centroid
    let centroid = V2D {
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

/// Define the cell groups that will exist in this experiment.
fn cell_groups(
    rng: &mut Pcg32,
    cq: &CharQuantities,
    num_cells: usize,
    randomization: bool,
) -> Vec<CellGroup> {
    vec![CellGroup {
        num_cells,
        layout: group_layout(num_cells, cq).unwrap(),
        parameters: gen_default_raw_params(rng, randomization)
            .gen_parameters(cq),
    }]
}

/// Generate CAL values between different cells.
fn gen_cal_mat() -> SymCcDat<f64> {
    SymCcDat::<f64>::new(2, 0.0)
}

/// Generate CIL values between different cells (see SI for
/// justification).
fn gen_cil_mat() -> SymCcDat<f64> {
    SymCcDat::<f64>::new(2, 60.0)
}

/// Generate raw world parameters, in particular, how
/// cells interact with each other, and any boundaries.
fn raw_world_parameters(
    coa_mag: Option<f64>,
    adh_mag: Option<f64>,
    cal_mag: Option<f64>,
    cil_mag: f64,
    char_quants: &CharQuantities,
) -> RawWorldParameters {
    // Some(RawCoaParams {
    //     los_penalty: 2.0,
    //     range: Length(100.0).micro(),
    //     mag: 100.0,
    // })
    let one_at = gen_default_phys_contact_dist();
    let coa = RawCoaParams::default_with_mag(coa_mag);
    let adh_mag = if let Some(x) = adh_mag {
        Some(gen_default_adhesion_mag(char_quants, x))
    } else {
        None
    };
    RawWorldParameters {
        vertex_eta: gen_default_vertex_viscosity(char_quants),
        interactions: RawInteractionParams {
            coa,
            chem_attr: None,
            bdry: None,
            phys_contact: RawPhysicalContactParams {
                range: RawCloseBounds::new(
                    one_at.mul_number(2.0),
                    one_at,
                ),
                adh_mag,
                cal_mag,
                cil_mag,
            },
        },
    }
}

/// Generate the experiment, so that it can be run.
pub fn generate(
    seed: Option<u64>,
    randomization: bool,
) -> Experiment {
    let mut rng = match seed {
        Some(s) => Pcg32::seed_from_u64(s),
        None => Pcg32::from_entropy(),
    };
    let cil = 60.0;
    let cal: Option<f64> = None;
    let adh: Option<f64> = None;
    let coa: Option<f64> = Some(0.0);

    let char_quants = gen_default_char_quants();
    let world_parameters =
        raw_world_parameters(coa, adh, cal, cil, &char_quants)
            .refine(&char_quants);
    let cell_groups =
        cell_groups(&mut rng, &char_quants, 4, randomization);

    //convert the option into string
    let cal = if let Some(i) = cal {
        i.to_string()
    } else {
        "None".to_string()
    };

    let adh = if let Some(i) = adh {
        i.to_string()
    } else {
        "None".to_string()
    };

    let coa = if let Some(i) = coa {
        i.to_string()
    } else {
        "None".to_string()
    };

    let seed_string = if let Some(i) = seed {
        i.to_string()
    } else {
        "None".to_string()
    };

    let random_string =
        if randomization == true { "rt" } else { "rf" };

    Experiment {
        file_name: format!(
            "four_cell_cil={}_cal={}_adh={}_coa={}_seed={}_{}",
            cil, cal, adh, coa, seed_string, random_string
        ),
        char_quants,
        world_parameters,
        cell_groups,
        rng,
        seed,
    }
}
