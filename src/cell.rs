// Copyright © 2020 Brian Merchant.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use crate::chemistry::{
    calc_conc_rgtps, calc_k_mem_offs, calc_k_mem_on, calc_kdgtps_rac, calc_kdgtps_rho,
    calc_kgtps_rac, calc_kgtps_rho, calc_net_fluxes, gen_rgtp_distrib, RgtpLayout,
};
use crate::consts::NVERTS;
use crate::math::{hill_function, P2D, max_f32, min_f32};
use crate::mechanics::{
    calc_cyto_forces, calc_edge_forces, calc_edge_lens, calc_edge_strains, calc_edge_unit_vecs,
    calc_global_strain, calc_rgtp_forces,
};
use crate::parameters::Parameters;
use crate::utils::circ_ix_minus;
use avro_schema_derive::Schematize;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use std::ops::{Add, Mul, Div, Sub};
use crate::rkdp5::{rkdp5, AuxArgs};

#[derive(Copy, Clone, Debug, Default, Deserialize, Serialize, Schematize)]
pub struct CellState {
    vertex_coords: [P2D; NVERTS as usize],
    rac_acts: [f32; NVERTS as usize],
    rac_inacts: [f32; NVERTS as usize],
    rho_acts: [f32; NVERTS as usize],
    rho_inacts: [f32; NVERTS as usize],
}

impl Add for CellState {
    type Output = CellState;

    fn add(self, rhs: CellState) -> CellState {
        let mut vertex_coords = [P2D::default(); NVERTS as usize];
        let mut rac_acts = [0.0_f32; NVERTS as usize];
        let mut rac_inacts = [0.0_f32; NVERTS as usize];
        let mut rho_acts = [0.0_f32; NVERTS as usize];
        let mut rho_inacts = [0.0_f32; NVERTS as usize];

        for i in 0..(NVERTS as usize) {
            vertex_coords[i] = &self.vertex_coords[i] + &rhs.vertex_coords[i];
            rac_acts[i] = &self.rac_acts[i] + &rhs.rac_acts[i];
            rac_inacts[i] = &self.rac_inacts[i] + &rhs.rac_inacts[i];
            rho_acts[i] = &self.rho_acts[i] + &rhs.rho_acts[i];
            rho_inacts[i] = &self.rho_inacts[i] + &rhs.rho_inacts[i]
        }

        Self::Output {
            vertex_coords,
            rac_acts,
            rac_inacts,
            rho_acts,
            rho_inacts,
        }
    }
}

impl Sub for CellState {
    type Output = CellState;

    fn sub(self, rhs: CellState) -> CellState {
        let mut vertex_coords = [P2D::default(); NVERTS as usize];
        let mut rac_acts = [0.0_f32; NVERTS as usize];
        let mut rac_inacts = [0.0_f32; NVERTS as usize];
        let mut rho_acts = [0.0_f32; NVERTS as usize];
        let mut rho_inacts = [0.0_f32; NVERTS as usize];

        for i in 0..(NVERTS as usize) {
            vertex_coords[i] = &self.vertex_coords[i] - &rhs.vertex_coords[i];
            rac_acts[i] = &self.rac_acts[i] - &rhs.rac_acts[i];
            rac_inacts[i] = &self.rac_inacts[i] - &rhs.rac_inacts[i];
            rho_acts[i] = &self.rho_acts[i] - &rhs.rho_acts[i];
            rho_inacts[i] = &self.rho_inacts[i] - &rhs.rho_inacts[i]
        }

        Self::Output {
            vertex_coords,
            rac_acts,
            rac_inacts,
            rho_acts,
            rho_inacts,
        }
    }
}

impl Add for &CellState {
    type Output = CellState;

    fn add(self, rhs: &CellState) -> CellState {
        let mut vertex_coords = [P2D::default(); NVERTS as usize];
        let mut rac_acts = [0.0_f32; NVERTS as usize];
        let mut rac_inacts = [0.0_f32; NVERTS as usize];
        let mut rho_acts = [0.0_f32; NVERTS as usize];
        let mut rho_inacts = [0.0_f32; NVERTS as usize];

        for i in 0..(NVERTS as usize) {
            vertex_coords[i] = &self.vertex_coords[i] + &rhs.vertex_coords[i];
            rac_acts[i] = &self.rac_acts[i] + &rhs.rac_acts[i];
            rac_inacts[i] = &self.rac_inacts[i] + &rhs.rac_inacts[i];
            rho_acts[i] = &self.rho_acts[i] + &rhs.rho_acts[i];
            rho_inacts[i] = &self.rho_inacts[i] + &rhs.rho_inacts[i]
        }

        Self::Output {
            vertex_coords,
            rac_acts,
            rac_inacts,
            rho_acts,
            rho_inacts,
        }
    }
}

impl Add<&CellState> for CellState {
    type Output = CellState;

    fn add(self, rhs: &CellState) -> CellState {
        let mut vertex_coords = [P2D::default(); NVERTS as usize];
        let mut rac_acts = [0.0_f32; NVERTS as usize];
        let mut rac_inacts = [0.0_f32; NVERTS as usize];
        let mut rho_acts = [0.0_f32; NVERTS as usize];
        let mut rho_inacts = [0.0_f32; NVERTS as usize];

        for i in 0..(NVERTS as usize) {
            vertex_coords[i] = &self.vertex_coords[i] + &rhs.vertex_coords[i];
            rac_acts[i] = &self.rac_acts[i] + &rhs.rac_acts[i];
            rac_inacts[i] = &self.rac_inacts[i] + &rhs.rac_inacts[i];
            rho_acts[i] = &self.rho_acts[i] + &rhs.rho_acts[i];
            rho_inacts[i] = &self.rho_inacts[i] + &rhs.rho_inacts[i]
        }

        Self::Output {
            vertex_coords,
            rac_acts,
            rac_inacts,
            rho_acts,
            rho_inacts,
        }
    }
}

impl Add<CellState> for &CellState {
    type Output = CellState;

    fn add(self, rhs: CellState) -> CellState {
        let mut vertex_coords = [P2D::default(); NVERTS as usize];
        let mut rac_acts = [0.0_f32; NVERTS as usize];
        let mut rac_inacts = [0.0_f32; NVERTS as usize];
        let mut rho_acts = [0.0_f32; NVERTS as usize];
        let mut rho_inacts = [0.0_f32; NVERTS as usize];

        for i in 0..(NVERTS as usize) {
            vertex_coords[i] = &self.vertex_coords[i] + &rhs.vertex_coords[i];
            rac_acts[i] = &self.rac_acts[i] + &rhs.rac_acts[i];
            rac_inacts[i] = &self.rac_inacts[i] + &rhs.rac_inacts[i];
            rho_acts[i] = &self.rho_acts[i] + &rhs.rho_acts[i];
            rho_inacts[i] = &self.rho_inacts[i] + &rhs.rho_inacts[i]
        }

        Self::Output {
            vertex_coords,
            rac_acts,
            rac_inacts,
            rho_acts,
            rho_inacts,
        }
    }
}


impl Div for CellState {
    type Output = CellState;

    fn div(self, rhs: CellState) -> CellState {
        let mut vertex_coords = [P2D::default(); NVERTS as usize];
        let mut rac_acts = [0.0_f32; NVERTS as usize];
        let mut rac_inacts = [0.0_f32; NVERTS as usize];
        let mut rho_acts = [0.0_f32; NVERTS as usize];
        let mut rho_inacts = [0.0_f32; NVERTS as usize];

        for i in 0..(NVERTS as usize) {
            vertex_coords[i] = self.vertex_coords[i] / rhs.vertex_coords[i];
            rac_acts[i] = &self.rac_acts[i] / &rhs.rac_acts[i];
            rac_inacts[i] = &self.rac_inacts[i] / &rhs.rac_inacts[i];
            rho_acts[i] = &self.rho_acts[i] / &rhs.rho_acts[i];
            rho_inacts[i] = &self.rho_inacts[i] / &rhs.rho_inacts[i]
        }

        Self::Output {
            vertex_coords,
            rac_acts,
            rac_inacts,
            rho_acts,
            rho_inacts,
        }
    }
}

impl Mul<CellState> for f32 {
    type Output = CellState;

    fn mul(self, rhs: CellState) -> CellState {
        let mut vertex_coords = [P2D::default(); NVERTS as usize];
        let mut rac_acts = [0.0_f32; NVERTS as usize];
        let mut rac_inacts = [0.0_f32; NVERTS as usize];
        let mut rho_acts = [0.0_f32; NVERTS as usize];
        let mut rho_inacts = [0.0_f32; NVERTS as usize];

        for i in 0..(NVERTS as usize) {
            vertex_coords[i] = self * rhs.vertex_coords[i];
            rac_acts[i] = self * &rhs.rac_acts[i];
            rac_inacts[i] = self * &rhs.rac_inacts[i];
            rho_acts[i] = self * &rhs.rho_acts[i];
            rho_inacts[i] = self * &rhs.rho_inacts[i]
        }

        Self::Output {
            vertex_coords,
            rac_acts,
            rac_inacts,
            rho_acts,
            rho_inacts,
        }
    }
}

impl Mul<&CellState> for f32 {
    type Output = CellState;

    fn mul(self, rhs: &CellState) -> CellState {
        let mut vertex_coords = [P2D::default(); NVERTS as usize];
        let mut rac_acts = [0.0_f32; NVERTS as usize];
        let mut rac_inacts = [0.0_f32; NVERTS as usize];
        let mut rho_acts = [0.0_f32; NVERTS as usize];
        let mut rho_inacts = [0.0_f32; NVERTS as usize];

        for i in 0..(NVERTS as usize) {
            vertex_coords[i] = self * rhs.vertex_coords[i];
            rac_acts[i] = self * &rhs.rac_acts[i];
            rac_inacts[i] = self * &rhs.rac_inacts[i];
            rho_acts[i] = self * &rhs.rho_acts[i];
            rho_inacts[i] = self * &rhs.rho_inacts[i]
        }

        Self::Output {
            vertex_coords,
            rac_acts,
            rac_inacts,
            rho_acts,
            rho_inacts,
        }
    }
}

impl Mul<&CellState> for &f32 {
    type Output = CellState;

    fn mul(self, rhs: &CellState) -> CellState {
        let mut vertex_coords = [P2D::default(); NVERTS as usize];
        let mut rac_acts = [0.0_f32; NVERTS as usize];
        let mut rac_inacts = [0.0_f32; NVERTS as usize];
        let mut rho_acts = [0.0_f32; NVERTS as usize];
        let mut rho_inacts = [0.0_f32; NVERTS as usize];

        for i in 0..(NVERTS as usize) {
            vertex_coords[i] = self * rhs.vertex_coords[i];
            rac_acts[i] = self * &rhs.rac_acts[i];
            rac_inacts[i] = self * &rhs.rac_inacts[i];
            rho_acts[i] = self * &rhs.rho_acts[i];
            rho_inacts[i] = self * &rhs.rho_inacts[i]
        }

        Self::Output {
            vertex_coords,
            rac_acts,
            rac_inacts,
            rho_acts,
            rho_inacts,
        }
    }
}


#[derive(Copy, Clone, Debug, Default, Deserialize, Serialize, Schematize)]
pub struct RacRandomState {
    x_rands: [f32; NVERTS as usize],
    next_updates: [u32; NVERTS as usize],
}

impl RacRandomState {
    fn init() -> RacRandomState {
        RacRandomState::default()
    }

    fn update(&self, _tstep: u32) -> RacRandomState {
        RacRandomState::default()
    }
}

#[derive(Copy, Clone, Debug, Default, Deserialize, Schematize, Serialize)]
pub struct InteractionState {
    x_cils: [f32; NVERTS as usize],
    x_chemoas: [f32; NVERTS as usize],
    x_coas: [f32; NVERTS as usize],
    x_rands: [f32; NVERTS as usize],
    x_bdrys: [f32; NVERTS as usize],
}

pub struct MechState {
    pub edge_strains: [f32; NVERTS as usize],
    pub rgtp_forces: [P2D; NVERTS as usize],
    pub cyto_forces: [P2D; NVERTS as usize],
    pub edge_forces: [P2D; NVERTS as usize],
}

pub struct ChemState {
    pub kdgtps_rac: [f32; NVERTS as usize],
    pub kgtps_rac: [f32; NVERTS as usize],
    pub rac_act_net_fluxes: [f32; NVERTS as usize],
    pub rac_inact_net_fluxes: [f32; NVERTS as usize],
    pub kdgtps_rho: [f32; NVERTS as usize],
    pub kgtps_rho: [f32; NVERTS as usize],
    pub rho_act_net_fluxes: [f32; NVERTS as usize],
    pub rho_inact_net_fluxes: [f32; NVERTS as usize],
    pub rac_mem_on: f32,
    pub rac_mem_offs: [f32; NVERTS as usize],
    pub rho_mem_on: f32,
    pub rho_mem_offs: [f32; NVERTS as usize],
}

pub struct GeomState {
    unit_edge_vecs: [P2D; NVERTS as usize],
    edge_lens: [f32; NVERTS as usize],
    unit_inward_vecs: [P2D; NVERTS as usize],
}

// pub struct ForceData {
//     rgtp_fs: Option<[P2D; NVERTS as usize]>,
//     edge_fs: Option<[P2D; NVERTS as usize]>,
//     cyto_fs: Option<[P2D; NVERTS as usize]>,
// }

fn increment_f32s(
    xs: &[f32; NVERTS as usize],
    dxs: &[f32; NVERTS as usize],
) -> [f32; NVERTS as usize] {
    let mut r = [0.0_f32; NVERTS as usize];
    (0..NVERTS as usize).for_each(|i| r[i] = xs[i] + dxs[i]);
    r
}

fn increment_vec2ds(
    xs: &[P2D; NVERTS as usize],
    dxs: &[P2D; NVERTS as usize],
) -> [P2D; NVERTS as usize] {
    let mut r = [P2D::default(); NVERTS as usize];
    (0..NVERTS as usize).for_each(|i| r[i] = xs[i] + dxs[i]);
    r
}

impl CellState {
    pub fn num_vars() -> usize {
        (NVERTS * 6) as usize
    }
    
    fn calc_geom_state(state: &CellState) -> GeomState {
        let uevs = calc_edge_unit_vecs(&state.vertex_coords);
        let edge_lens: [f32; NVERTS as usize] = calc_edge_lens(&uevs);
        let mut uivs = [P2D::default(); NVERTS as usize];
        (0..NVERTS as usize).for_each(|i| {
            let im1 = circ_ix_minus(i as usize, NVERTS as usize);
            let tangent = (uevs[i] + uevs[im1]).unitize();
            uivs[i] = tangent.normal();
        });

        GeomState {
            unit_edge_vecs: uevs,
            edge_lens,
            unit_inward_vecs: uivs,
        }
    }

    fn calc_mech_state(state: &CellState, geom_state: &GeomState, parameters: &Parameters) -> MechState {
        let GeomState {
            unit_edge_vecs: uevs,
            edge_lens,
            unit_inward_vecs: uivs,
        } = geom_state;
        let rgtp_forces = calc_rgtp_forces(
            &state.rac_acts,
            &state.rho_acts,
            uivs,
            parameters.halfmax_vertex_rgtp_act,
            parameters.const_protrusive,
            parameters.const_retractive,
        );
        let cyto_forces = calc_cyto_forces(
            &state.vertex_coords,
            &uivs,
            parameters.rest_area,
            parameters.stiffness_ctyo,
        );
        let edge_strains = calc_edge_strains(&edge_lens, parameters.rest_edge_len);
        let edge_forces = calc_edge_forces(&edge_strains, uevs, parameters.stiffness_edge);

        MechState {
            edge_strains,
            rgtp_forces,
            cyto_forces,
            edge_forces,
        }
    }

    pub fn calc_chem_state(state: &CellState, geom_state: &GeomState, inter_state: &InteractionState, parameters: &Parameters) -> ChemState {
        let GeomState { edge_lens, .. } = geom_state;
        let mut avg_edge_lens: [f32; NVERTS as usize] = [0.0_f32; NVERTS as usize];
        (0..NVERTS as usize).for_each(|i| {
            let im1 = circ_ix_minus(i as usize, NVERTS as usize);
            avg_edge_lens[i] = (edge_lens[i] + edge_lens[im1]) / 2.0;
        });

        let conc_rac_acts = calc_conc_rgtps(&avg_edge_lens, &state.rac_acts);
        let conc_rac_inacts = calc_conc_rgtps(&avg_edge_lens, &state.rac_inacts);
        let conc_rho_acts = calc_conc_rgtps(&avg_edge_lens, &state.rho_acts);
        let conc_rho_inacts = calc_conc_rgtps(&avg_edge_lens, &state.rho_inacts);

        let kgtps_rac = calc_kgtps_rac(
            &state.rac_acts,
            &conc_rac_acts,
            &inter_state.x_rands,
            &inter_state.x_coas,
            &inter_state.x_chemoas,
            parameters.kgtp_rac,
            parameters.kgtp_rac_auto,
            parameters.halfmax_vertex_rgtp_conc,
        );
        let x_tens = hill_function(
            parameters.halfmax_tension_inhib,
            calc_global_strain(&edge_lens, parameters.rest_edge_len, NVERTS),
        );
        let kdgtps_rac = calc_kdgtps_rac(
            &state.rac_acts,
            &conc_rac_acts,
            &inter_state.x_cils,
            x_tens,
            parameters.kdgtp_rac,
            parameters.kdgtp_rho_on_rac,
            parameters.halfmax_vertex_rgtp_conc,
        );
        let kgtps_rho = calc_kgtps_rho(
            &state.rho_acts,
            &conc_rac_acts,
            &inter_state.x_cils,
            parameters.kgtp_rho,
            parameters.halfmax_vertex_rgtp_conc,
            parameters.kgtp_rho_auto,
        );
        let kdgtps_rho = calc_kdgtps_rho(
            &state.rho_acts,
            &conc_rac_acts,
            parameters.kdgtp_rho,
            parameters.kdgtp_rac_on_rho,
            parameters.halfmax_vertex_rgtp_conc,
        );
        let rac_act_net_fluxes = calc_net_fluxes(
            &edge_lens,
            &avg_edge_lens,
            parameters.diffusion_rgtp,
            &conc_rac_acts,
        );
        let rho_act_net_fluxes = calc_net_fluxes(
            &edge_lens,
            &avg_edge_lens,
            parameters.diffusion_rgtp,
            &conc_rho_acts,
        );
        let rac_inact_net_fluxes = calc_net_fluxes(
            &edge_lens,
            &avg_edge_lens,
            parameters.diffusion_rgtp,
            &conc_rac_inacts,
        );
        let rho_inact_net_fluxes = calc_net_fluxes(
            &edge_lens,
            &avg_edge_lens,
            parameters.diffusion_rgtp,
            &conc_rho_inacts,
        );

        let rac_cyto =
            1.0_f32 - state.rac_acts.iter().sum::<f32>() - state.rac_inacts.iter().sum::<f32>();
        let rac_mem_on = calc_k_mem_on(rac_cyto, parameters.k_mem_on);
        let rac_mem_offs = calc_k_mem_offs(&state.rho_inacts, parameters.k_mem_off);
        let rho_cyto =
            1.0 - state.rho_acts.iter().sum::<f32>() - state.rho_inacts.iter().sum::<f32>();
        let rho_mem_on = calc_k_mem_on(rho_cyto, parameters.k_mem_on);
        let rho_mem_offs = calc_k_mem_offs(&state.rac_inacts, parameters.k_mem_off);

        ChemState {
            kdgtps_rac,
            kgtps_rac,
            rac_act_net_fluxes,
            rac_inact_net_fluxes,
            kdgtps_rho,
            kgtps_rho,
            rho_act_net_fluxes,
            rho_inact_net_fluxes,
            rac_mem_on,
            rac_mem_offs,
            rho_mem_on,
            rho_mem_offs,
        }
    }
    
    fn dynamics_f(dt: f32, state: &CellState, inter_state: &InteractionState, params: &Parameters) -> CellState {
        let geom_state = Self::calc_geom_state(state);
        let chem_state = Self::calc_chem_state(&state, &geom_state, inter_state, params);
        let mech_state = Self::calc_mech_state(&state, &geom_state, params);

        let mut delta_rac_acts = [0.0_f32; NVERTS as usize];
        let mut delta_rac_inacts = [0.0_f32; NVERTS as usize];
        let mut delta_rho_acts = [0.0_f32; NVERTS as usize];
        let mut delta_rho_inacts = [0.0_f32; NVERTS as usize];
        let mut delta_pos = [P2D::default(); NVERTS as usize];
        for i in 0..NVERTS as usize {
            let inactivated_rac = chem_state.kdgtps_rac[i] * state.rac_acts[i];
            let activated_rac = chem_state.kgtps_rac[i] * state.rac_inacts[i];
            let delta_rac_activated = activated_rac - inactivated_rac;
            delta_rac_acts[i] = (delta_rac_activated + chem_state.rac_act_net_fluxes[i]) * dt;
            delta_rac_inacts[i] =
                (-1.0 * delta_rac_activated + chem_state.rac_inact_net_fluxes[i] + chem_state.rac_mem_on
                    - chem_state.rac_mem_offs[i])
                    * dt;

            let inactivated_rho = chem_state.kdgtps_rho[i] * state.rho_acts[i];
            let activated_rho = chem_state.kgtps_rho[i] * state.rho_inacts[i];
            let delta_rho_activated = activated_rho - inactivated_rho;
            delta_rho_acts[i] = (delta_rho_activated + chem_state.rho_act_net_fluxes[i]) * dt;
            delta_rho_inacts[i] =
                (-1.0 * delta_rho_activated + chem_state.rho_inact_net_fluxes[i] + chem_state.rho_mem_on
                    - chem_state.rho_mem_offs[i])
                    * dt;

            //println!("rgtp_f: {:?}", rgtp_forces[i]);
            //println!("cyto_f: {:?}", cyto_forces[i]);
            //println!("edge_f: {:?}", edge_forces[i] - edge_forces[circ_ix_minus(i, NVERTS)]);
            let sum_f = mech_state.rgtp_forces[i] + mech_state.cyto_forces[i] + mech_state.edge_forces[i]
                - mech_state.edge_forces[circ_ix_minus(i as usize, NVERTS as usize)];
            //println!("sum_f: {:?}", sum_f);
            //println!("sum_f * dt / eta: {:?}", sum_f.scalar_mul(dt / parameters.vertex_eta));
            delta_pos[i] = (dt / params.vertex_eta) * sum_f;
        }

        let vertex_coords = increment_vec2ds(&state.vertex_coords, &delta_pos);
        let rac_acts = increment_f32s(&state.rac_acts, &delta_rac_acts);
        let rac_inacts = increment_f32s(&state.rac_inacts, &delta_rac_inacts);
        let rho_acts = increment_f32s(&state.rho_acts, &delta_rho_acts);
        let rho_inacts = increment_f32s(&state.rho_inacts, &delta_rho_inacts);

        CellState::new(vertex_coords, rac_acts, rac_inacts, rho_acts, rho_inacts)
    }
    
    // pub fn decompress_force_data(
    //     &self,
    //     rgtp_forces: bool,
    //     edge_forces: bool,
    //     cyto_forces: bool,
    //     parameters: &Parameters,
    // ) -> ForceData {
    //     let (rgtp_fs, edge_fs, cyto_fs) = if rgtp_forces || edge_forces {
    //         let pg = self.calc_geom_vars();
    //         let rfs = if rgtp_forces {
    //             Some(calc_rgtp_forces(
    //                 &self.rac_acts,
    //                 &self.rho_acts,
    //                 &pg.unit_inward_vecs,
    //                 parameters.halfmax_vertex_rgtp_act,
    //                 parameters.const_protrusive,
    //                 parameters.const_retractive,
    //             ))
    //         } else {
    //             None
    //         };
    //
    //         let efs = if edge_forces {
    //             let edge_strains =
    //                 calc_edge_strains(&pg.edge_lens, parameters.rest_edge_len);
    //             Some(calc_edge_forces(
    //                 &edge_strains,
    //                 &pg.unit_edge_vecs,
    //                 parameters.stiffness_edge,
    //             ))
    //         } else {
    //             None
    //         };
    //
    //         let cfs = if cyto_forces {
    //             Some(calc_cyto_forces(
    //                 &self.vertex_coords,
    //                 &pg.unit_inward_vecs,
    //                 parameters.rest_area,
    //                 parameters.stiffness_ctyo,
    //             ))
    //         } else {
    //             None
    //         };
    //
    //         (rfs, efs, cfs)
    //     } else {
    //         (None, None, None)
    //     };
    //
    //     ForceData {
    //         rgtp_fs,
    //         edge_fs,
    //         cyto_fs,
    //     }
    // }

    pub fn new(
        vertex_coords: [P2D; NVERTS as usize],
        rac_acts: [f32; NVERTS as usize],
        rac_inacts: [f32; NVERTS as usize],
        rho_acts: [f32; NVERTS as usize],
        rho_inacts: [f32; NVERTS as usize],
    ) -> CellState {
        // x_cils: [f32; NVERTS], x_coas: [f32; NVERTS], x_chemoas: [f32; NVERTS], x_rands: [f32; NVERTS], x_bdrys: [f32; NVERTS as usize];
        CellState {
            vertex_coords,
            rac_acts,
            rac_inacts,
            rho_acts,
            rho_inacts,
        }
    }

    pub fn scalar_mul(&self, s: f32) -> CellState {
        let mut vertex_coords = [P2D::default(); NVERTS as usize];
        let mut rac_acts = [0.0_f32; NVERTS as usize];
        let mut rac_inacts = [0.0_f32; NVERTS as usize];
        let mut rho_acts = [0.0_f32; NVERTS as usize];
        let mut rho_inacts = [0.0_f32; NVERTS as usize];

        for i in 0..(NVERTS as usize) {
            vertex_coords[i] = s * self.vertex_coords[i];
            rac_acts[i] = self.rac_acts[i] * s;
            rac_inacts[i] = self.rac_inacts[i] * s;
            rho_acts[i] = self.rho_acts[i] * s;
            rho_inacts[i] = self.rho_inacts[i] * s;
        }

        CellState {
            vertex_coords,
            rac_acts,
            rac_inacts,
            rho_acts,
            rho_inacts
        }
    }

    pub fn scalar_add(&self, s: f32) -> CellState {
        let mut vertex_coords = [P2D::default(); NVERTS as usize];
        let mut rac_acts = [0.0_f32; NVERTS as usize];
        let mut rac_inacts = [0.0_f32; NVERTS as usize];
        let mut rho_acts = [0.0_f32; NVERTS as usize];
        let mut rho_inacts = [0.0_f32; NVERTS as usize];

        for i in 0..(NVERTS as usize) {
            vertex_coords[i] = s + self.vertex_coords[i];
            rac_acts[i] = self.rac_acts[i] + s;
            rac_inacts[i] = self.rac_inacts[i] + s;
            rho_acts[i] = self.rho_acts[i] + s;
            rho_inacts[i] = self.rho_inacts[i] + s;
        }

        CellState {
            vertex_coords,
            rac_acts,
            rac_inacts,
            rho_acts,
            rho_inacts
        }
    }
    
    pub fn abs(&self) -> CellState {
        let mut vertex_coords = [P2D::default(); NVERTS as usize];
        let mut rac_acts = [0.0_f32; NVERTS as usize];
        let mut rac_inacts = [0.0_f32; NVERTS as usize];
        let mut rho_acts = [0.0_f32; NVERTS as usize];
        let mut rho_inacts = [0.0_f32; NVERTS as usize];

        for i in 0..(NVERTS as usize) {
            vertex_coords[i] = vertex_coords[i].abs();
            rac_acts[i] = self.rac_acts[i].abs();
            rac_inacts[i] = self.rac_inacts[i].abs();
            rho_acts[i] = self.rho_acts[i].abs();
            rho_inacts[i] = self.rho_inacts[i].abs();
        }
        
        CellState {
            vertex_coords,
            rac_acts,
            rac_inacts,
            rho_acts,
            rho_inacts
        }
    }

    pub fn powi(&self, x: i32) -> CellState {
        let mut vertex_coords = [P2D::default(); NVERTS as usize];
        let mut rac_acts = [0.0_f32; NVERTS as usize];
        let mut rac_inacts = [0.0_f32; NVERTS as usize];
        let mut rho_acts = [0.0_f32; NVERTS as usize];
        let mut rho_inacts = [0.0_f32; NVERTS as usize];

        for i in 0..(NVERTS as usize) {
            vertex_coords[i] = vertex_coords[i].powi(x);
            rac_acts[i] = self.rac_acts[i].powi(x);
            rac_inacts[i] = self.rac_inacts[i].powi(x);
            rho_acts[i] = self.rho_acts[i].powi(x);
            rho_inacts[i] = self.rho_inacts[i].powi(x);
        }

        CellState {
            vertex_coords,
            rac_acts,
            rac_inacts,
            rho_acts,
            rho_inacts
        }
    }

    pub fn max(&self, other: &CellState) -> CellState {
        let mut vertex_coords = [P2D::default(); NVERTS as usize];
        let mut rac_acts = [0.0_f32; NVERTS as usize];
        let mut rac_inacts = [0.0_f32; NVERTS as usize];
        let mut rho_acts = [0.0_f32; NVERTS as usize];
        let mut rho_inacts = [0.0_f32; NVERTS as usize];

        for i in 0..(NVERTS as usize) {
            vertex_coords[i] = vertex_coords[i].max(&other.vertex_coords[i]);
            rac_acts[i] = max_f32(self.rac_acts[i], other.rac_acts[i]);
            rac_inacts[i] = max_f32(self.rac_inacts[i], other.rac_inacts[i]);
            rho_acts[i] = max_f32(self.rho_acts[i], other.rho_acts[i]);
            rho_inacts[i] = max_f32(self.rho_inacts[i], other.rho_inacts[i]);
        }

        CellState {
            vertex_coords,
            rac_acts,
            rac_inacts,
            rho_acts,
            rho_inacts
        }
    }

    pub fn min(&self, other: &CellState) -> CellState {
        let mut vertex_coords = [P2D::default(); NVERTS as usize];
        let mut rac_acts = [0.0_f32; NVERTS as usize];
        let mut rac_inacts = [0.0_f32; NVERTS as usize];
        let mut rho_acts = [0.0_f32; NVERTS as usize];
        let mut rho_inacts = [0.0_f32; NVERTS as usize];

        for i in 0..(NVERTS as usize) {
            vertex_coords[i] = vertex_coords[i].min(&other.vertex_coords[i]);
            rac_acts[i] = min_f32(self.rac_acts[i], other.rac_acts[i]);
            rac_inacts[i] = min_f32(self.rac_inacts[i], other.rac_inacts[i]);
            rho_acts[i] = min_f32(self.rho_acts[i], other.rho_acts[i]);
            rho_inacts[i] = min_f32(self.rho_inacts[i], other.rho_inacts[i]);
        }

        CellState {
            vertex_coords,
            rac_acts,
            rac_inacts,
            rho_acts,
            rho_inacts
        }
    }

    pub fn sum(&self) -> f32 {
        let mut r: f32 = 0.0;

        for i in 0..(NVERTS as usize) {
            r += self.vertex_coords[i].x + self.vertex_coords[i].y;
            r += self.rac_acts[i];
            r += self.rac_inacts[i];
            r += self.rho_acts[i];
            r += self.rho_inacts[i];
        }

        r
    }

    pub fn average(&self) -> f32 {
        self.sum()/(Self::num_vars() as f32)
    }
}

/// Model cell.
#[derive(Clone, Deserialize, Serialize, Schematize)]
pub struct Cell {
    /// Cell index.
    pub ix: u32,
    /// Index of cell type.
    pub group_ix: u32,
    // /// If `true`, `evolve` does not change current cell state.
    // skip_dynamics: bool,
    pub state: CellState,
    pub interactions: InteractionState,
    pub rac_randomization: RacRandomState,
}

fn gen_vertex_coords(centroid: P2D, radius: f32) -> [P2D; NVERTS as usize] {
    let mut r = [P2D::default(); NVERTS as usize];
    (0..NVERTS as usize).for_each(|vix| {
        let vf = (vix as f32) / (NVERTS as f32);
        let theta = 2.0 * PI * vf;
        r[vix] = P2D {
            x: centroid.x + theta.cos() * radius,
            y: centroid.y + theta.sin() * radius,
        };
    });
    r
}

impl Cell {
    pub fn new(ix: u32, group_ix: u32, parameters: &Parameters, centroid: P2D) -> Cell {
        let vertex_coords = gen_vertex_coords(centroid, parameters.cell_r);
        let (rac_acts, rac_inacts) = gen_rgtp_distrib(
            parameters.init_frac_active,
            parameters.init_frac_inactive,
            &RgtpLayout::Random,
        );

        let (rho_acts, rho_inacts) = gen_rgtp_distrib(
            parameters.init_frac_active,
            parameters.init_frac_inactive,
            &RgtpLayout::Random,
        );

        Cell {
            ix,
            group_ix,
            state: CellState::new(
                vertex_coords,
                rac_acts,
                rac_inacts,
                rho_acts,
                rho_inacts,
            ),
            interactions: InteractionState::default(),
            rac_randomization: RacRandomState::init(),
        }
    }

    pub fn simulate_euler(&self, tstep: u32, parameters: &Parameters) -> Cell {
        let mut state = self.state.clone();
        let nsteps: u32 = 10000;
        let dt = 1.0 / (nsteps as f32);
        for i in 0..nsteps {
            state = CellState::dynamics_f(dt, &state, &self.interactions, parameters);
            #[cfg(debug_assertions)]
            println!("step: {}", i);
            #[cfg(debug_assertions)]
            println!("state: {:?}", state);
        }
        println!("state: {:?}", state);
        Cell {
            ix: self.ix,
            group_ix: self.group_ix,
            state,
            interactions: self.interactions.clone(),
            rac_randomization: self.rac_randomization.update(tstep),
        }
    }

    pub fn simulate_rkdp5(&self, tstep: u32, parameters: &Parameters) -> Cell {
        let aux_args = AuxArgs {
            max_iters: 100000,
            atol: 1e-6,
            rtol: 0.0,
            init_h_factor: Some(0.01)
        };
        let result = rkdp5(1.0, CellState::dynamics_f, self.state, &self.interactions, parameters, aux_args);

        println!("num_iters: {}, num_rejections: {}", result.num_iters, result.num_rejections);
        let state = result.y.expect("too many iterations!");
        println!("state: {:?}", state);
        Cell {
            ix: self.ix,
            group_ix: self.group_ix,
            state,
            interactions: self.interactions.clone(),
            rac_randomization: self.rac_randomization.update(tstep),
        }
    }
}
