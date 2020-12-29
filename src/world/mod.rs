// Copyright © 2020 Brian Merchant.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
pub mod hardio;
#[cfg(feature = "custom_debug")]
use crate::cell::confirm_volume_exclusion;
use crate::cell::core_state::CoreState;
use crate::cell::CellState;
use crate::experiments::{CellGroup, Experiment};
use crate::interactions::{
    CellInteractions, InteractionGenerator, RgtpState,
};
use crate::math::v2d::{poly_to_string, V2D};
use crate::parameters::{Parameters, WorldParameters};
use crate::world::hardio::{save_data, save_schema};
use crate::NVERTS;
use avro_rs::Writer;
use avro_schema_derive::Schematize;
use once_cell::unsync::Lazy;
//use rand_core::SeedableRng;
use rand::seq::SliceRandom;
use rand::{RngCore, SeedableRng};
use rand_distr::{Distribution, Normal};
use rand_pcg::Pcg64;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use std::path::PathBuf;

#[derive(Clone, Deserialize, Serialize, Schematize)]
pub struct Cells {
    pub tstep: u32,
    pub cell_states: Vec<CellState>,
    pub interactions: Vec<CellInteractions>,
}

impl Cells {
    #[cfg(not(feature = "custom_debug"))]
    fn simulate(
        &self,
        rng: &mut Pcg64,
        cell_regs: &mut [Option<RandomEventGenerator>],
        world_parameters: &WorldParameters,
        group_parameters: &[Parameters],
        interaction_generator: &mut InteractionGenerator,
    ) -> Cells {
        let mut cells =
            vec![self.cell_states[0]; self.cell_states.len()];
        let shuffled_cells = {
            let mut crs =
                self.cell_states.iter().collect::<Vec<&CellState>>();
            crs.shuffle(rng);
            crs
        };
        for c in shuffled_cells {
            // println!("------------------");
            let ci = c.ix as usize;
            // println!("ci: {}", ci);
            // println!(
            //     "contacts: {:?}",
            //     interaction_generator.get_physical_contacts(ci)
            // );
            let contact_polys =
                interaction_generator.get_contact_data(ci);
            let new_cell_state = c.simulate_euler(
                self.tstep,
                &self.interactions[ci],
                contact_polys,
                cell_regs[ci].as_mut(),
                world_parameters,
                &group_parameters[c.group_ix as usize],
            );
            // println!("----------------------");
            interaction_generator
                .update(ci, &new_cell_state.state.vertex_coords);
            cells[ci] = new_cell_state;
        }
        Cells {
            tstep: self.tstep + 1,
            cell_states: cells,
            interactions: interaction_generator.generate(),
        }
    }

    #[cfg(feature = "custom_debug")]
    fn simulate(
        &self,
        tstep: u32,
        rng: &mut Pcg64,
        cell_regs: &mut [Option<RandomEventGenerator>],
        world_parameters: &WorldParameters,
        group_parameters: &[Parameters],
        interaction_generator: &mut InteractionGenerator,
    ) -> Result<Cells, String> {
        let mut new_cell_states =
            vec![self.cell_states[0]; self.cell_states.len()];
        let shuffled_cells = {
            let mut crs =
                self.cell_states.iter().collect::<Vec<&CellState>>();
            crs.shuffle(rng);
            crs
        };
        for cell_state in shuffled_cells {
            // println!("------------------");
            let ci = cell_state.ix as usize;
            // println!("ci: {}", ci);
            // println!(
            //     "contacts: {:?}",
            //     interaction_generator.get_physical_contacts(ci)
            // );
            // let contacts =
            //     interaction_generator.get_physical_contacts(ci);
            let contact_data =
                interaction_generator.get_contact_data(ci);
            let vi: usize = if ci == 0 { 4 } else { 12 };
            // println!("----------------------------");
            // println!(
            //     "    ci = {}; vi = {}; x_adh = {}; f_tot: {}",
            //     ci,
            //     vi,
            //     self.interactions[ci].x_adhs[vi].mag(),
            //     cell_state.mech.sum_fs[vi].mag(),
            // );
            let new_cell_state = cell_state.simulate_rkdp5(
                self.tstep,
                &self.interactions[ci],
                contact_data,
                cell_regs[ci].as_mut(),
                world_parameters,
                &group_parameters[cell_state.group_ix as usize],
            )?;
            // println!(
            //     "                      old_v = {},\n                      new_v = {}, \n                      delta_mag: {}",
            //     cell_state.core.vertex_coords[vi],
            //     new_cell_state.core.vertex_coords[vi],
            //     (cell_state.core.vertex_coords[vi]
            //         - new_cell_state.core.vertex_coords[vi])
            //         .mag()
            // );
            // println!("----------------------------");
            // if (tstep == 347 && ci == 1) || (tstep == 346 && ci == 0)
            // {
            //     println!(
            //         "cell{} at {}->{}: {}",
            //         ci,
            //         tstep,
            //         tstep + 1,
            //         poly_to_string(
            //             &new_cell_state.state.vertex_coords
            //         )
            //     );
            // }
            // println!("----------------------");
            interaction_generator
                .update(ci, &new_cell_state.core.poly);
            confirm_volume_exclusion(
                &new_cell_state.core.poly,
                &interaction_generator.get_contact_data(ci),
                &format!("world cell {}", ci),
            )?;
            new_cell_states[ci] = new_cell_state;
        }
        Ok(Cells {
            tstep: self.tstep + 1,
            cell_states: new_cell_states,
            interactions: interaction_generator.generate(),
        })
    }
}

pub struct RandomEventGenerator {
    pub(crate) rng: Pcg64,
    distrib: Normal<f32>,
}

impl RandomEventGenerator {
    pub fn sample(&mut self) -> f32 {
        self.distrib.sample(&mut self.rng)
    }
}

pub struct World {
    tstep_length: f32,
    global_params: WorldParameters,
    cell_group_params: Vec<Parameters>,
    pub history: Vec<Cells>,
    cell_regs: Vec<Option<RandomEventGenerator>>,
    cell_states: Cells,
    interaction_generator: InteractionGenerator,
    pub rng: Pcg64,
    output_dir: PathBuf,
    exp_name: String,
}

fn gen_poly(centroid: &V2D, radius: f32) -> [V2D; NVERTS] {
    let mut r = [V2D::default(); NVERTS];
    (0..NVERTS).for_each(|vix| {
        let vf = (vix as f32) / (NVERTS as f32);
        let theta = 2.0 * PI * vf;
        r[vix] = V2D {
            x: centroid.x + theta.cos() * radius,
            y: centroid.y + theta.sin() * radius,
        };
    });
    r
}

impl World {
    pub fn new(experiment: Experiment, output_dir: PathBuf) -> World {
        // Unpack relevant info from `Experiment` data structure.
        let Experiment {
            char_quants,
            world_parameters,
            cell_groups,
            mut rng,
            ..
        } = experiment;
        // Extract the parameters from each `CellGroup` object obtained
        // from the `Experiment`.
        let group_parameters = cell_groups
            .iter()
            .map(|cg| cg.parameters.clone())
            .collect::<Vec<Parameters>>();

        // Create a list of indices of the groups. and reate a vector
        // of the cell centroids in each group.
        let mut cell_group_ixs = vec![];
        let mut cell_centroids = vec![];
        cell_groups.iter().enumerate().for_each(|(gix, cg)| {
            cell_group_ixs
                .append(&mut vec![gix; cg.num_cells as usize]);
            cell_centroids
                .append(&mut gen_cell_centroids(cg).unwrap())
        });
        // Generate the cell polygons from the cell centroid
        // information generated in the last step.
        let cell_polys = cell_group_ixs
            .iter()
            .zip(cell_centroids.iter())
            .map(|(&gix, cc)| {
                gen_poly(cc, group_parameters[gix].cell_r)
            })
            .collect::<Vec<[V2D; NVERTS]>>();
        // Create initial cell states, using the parameters associated
        // with the cell a cell group is in, and the cell's centroid
        // location.
        let cell_core_states = cell_group_ixs
            .iter()
            .zip(cell_polys.iter())
            .map(|(&gix, poly)| {
                let parameters = &group_parameters[gix];
                CoreState::new(
                    *poly,
                    parameters.init_rac,
                    parameters.init_rho,
                )
            })
            .collect::<Vec<CoreState>>();
        // Create initial Rho GTPase distributions.
        let cell_rgtps = cell_group_ixs
            .iter()
            .zip(cell_core_states.iter())
            .map(|(&gix, state)| {
                let parameters = &group_parameters[gix];
                state.calc_rgtp_state(parameters)
            })
            .collect::<Vec<[RgtpState; NVERTS]>>();
        // Create a new `InteractionGenerator`.
        let interact_gen = InteractionGenerator::new(
            &cell_polys,
            &cell_rgtps,
            world_parameters.interactions,
        );
        // Generate initial cell interactions.
        let cell_interactions = interact_gen.generate();
        // Create `Cell` structures to represent each cell.
        let mut cells = vec![];
        let mut cell_regs = vec![];
        for (cell_ix, group_ix) in
            cell_group_ixs.into_iter().enumerate()
        {
            // Parameters that will be used by this cell. Determined
            // by figuring out which group it belongs to, as all cells
            // within a group use the same parameters.
            let parameters = &group_parameters[group_ix];
            // Create a function which generates normally distributed
            // random variables with mean `rand_avg_t` and std
            // `rand_std_t`. Used to determine time steps between
            // Rac1 randomization events.
            let normal = Lazy::new(|| {
                Normal::new(
                    parameters.rand_avg_t,
                    parameters.rand_std_t,
                )
                .unwrap()
            });
            // Create a random event generator for this cell. Will be
            // `None` if randomization is turned off.
            let mut creg = if parameters.randomization {
                Some(RandomEventGenerator {
                    rng: Pcg64::seed_from_u64(rng.next_u64()),
                    distrib: *normal,
                })
            } else {
                None
            };
            // Create a new cell.
            cells.push(CellState::new(
                cell_ix as u32,
                group_ix as u32,
                cell_core_states[cell_ix],
                &cell_interactions[cell_ix],
                parameters,
                creg.as_mut(),
            ));
            cell_regs.push(creg);
        }
        let state = Cells {
            tstep: 0,
            cell_states: cells,
            interactions: cell_interactions.clone(),
        };
        let history = vec![state.clone()];
        World {
            tstep_length: char_quants.time(),
            global_params: world_parameters,
            cell_group_params: group_parameters,
            cell_regs,
            history,
            cell_states: state,
            interaction_generator: interact_gen,
            rng,
            output_dir,
            exp_name: experiment.title,
        }
    }

    pub fn simulate(&mut self, final_tpoint: f32) {
        let num_tsteps =
            (final_tpoint / self.tstep_length).ceil() as u32;
        while self.cell_states.tstep < num_tsteps {
            // println!(
            //     "tstep: {}/{}",
            //     self.cell_states.tstep, num_tsteps
            // );
            let tstep = self.cell_states.tstep;
            #[cfg(feature = "custom_debug")]
            let new_state: Cells = self
                .cell_states
                .simulate(
                    tstep,
                    &mut self.rng,
                    self.cell_regs.as_mut_slice(),
                    &self.global_params,
                    &self.cell_group_params,
                    &mut self.interaction_generator,
                )
                .unwrap_or_else(|e| {
                    self.save_history();
                    panic!(
                        "tstep: {}\n{}",
                        self.cell_states.tstep, e
                    );
                });

            #[cfg(not(feature = "custom_debug"))]
            let new_state = self.cell_states.simulate(
                &mut self.rng,
                self.cell_regs.as_mut_slice(),
                &self.global_params,
                &self.cell_group_params,
                &mut self.interaction_generator,
            );

            self.history.push(new_state.clone());
            self.cell_states = new_state;
        }
    }

    pub fn save_history(&self) {
        #[cfg(feature = "custom_debug")]
        let name = format!("history_dbg_{}", &self.exp_name);
        #[cfg(not(feature = "custom_debug"))]
        let name = format!("history_{}", &self.exp_name);

        let schema = Cells::schematize(None);
        save_schema(&name, &schema, &self.output_dir);
        let mut writer = Writer::new(&schema, Vec::new()); //Writer::with_codec(&self.state_schema, Vec::new(), avro_rs::Codec::Deflate);
        for ws in self.history.iter() {
            writer.append_ser(ws).unwrap();
        }
        let encoded = writer.into_inner().unwrap();
        save_data(&name, &encoded, &self.output_dir);
    }
}

fn gen_cell_centroids(cg: &CellGroup) -> Result<Vec<V2D>, String> {
    let CellGroup {
        num_cells,
        layout,
        parameters,
    } = cg;
    let cell_r = parameters.cell_r;
    if layout.width * layout.height >= *num_cells {
        let mut r = vec![];
        let first_cell_centroid = V2D {
            x: layout.bottom_left.x + cell_r,
            y: layout.bottom_left.y + cell_r,
        };
        let row_delta = V2D {
            x: 0.0,
            y: 2.0 * cell_r,
        };
        let col_delta = V2D {
            x: 2.0 * cell_r,
            y: 0.0,
        };
        for ix in 0..*num_cells {
            let row = ix / layout.width;
            let col = ix - layout.width * row;
            let cg = first_cell_centroid
                + (row as f32) * row_delta
                + (col as f32) * col_delta;
            r.push(cg);
        }
        Ok(r)
    } else {
        Err(format!(
            "Cell group layout area ({}x{}) not large enough to fit {} cells.",
            layout.width, layout.height, num_cells
        ))
    }
}
