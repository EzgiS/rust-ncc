import matplotlib.pyplot as plt
import json
import numpy as np
import json
import cbor2

from python_scripts.plot_2 import sum_non_adh_forces_per_cell_per_tstep, rgtp_forces_per_cell_per_tstep, \
    edge_forces_per_cell_per_tstep, cyto_forces_per_cell_per_tstep

output = None

file_name = "../output/separated_pair_cil=30_cal=None_adh=10_coa=24_seed=7_rt.cbor"


snapshots = []
with open(file_name, mode='rb') as sf:
    world_history = cbor2.load(sf)
    success = True
    while success:
        try:
            snapshots += cbor2.load(sf)
        except:
            success = False

tsteps = [s["tstep"] for s in snapshots]
state_recs = [s["cells"] for s in snapshots]
frequency = world_history["snap_freq"]


def lookup_tstep_ix(tstep):
    return int(np.floor(tstep/frequency))

def p2ds_to_numpy(p2ds):
    vs = []
    for p2d in p2ds:
        vs.append([p2d['x'], p2d['y']])
    return np.array(vs)


def extract_p2ds_from_cell_states(state_key, dat_key, state_recs):
    dat_per_cell_per_tstep = []
    for rec in state_recs:
        dat_per_cell = []
        for cell_rec in rec['states']:
            dat_per_cell.append(p2ds_to_numpy(cell_rec[state_key][dat_key]))
        dat_per_cell_per_tstep.append(np.array(dat_per_cell))
    return np.array(dat_per_cell_per_tstep)

def extract_p2ds_from_interactions(dat_key, state_recs):
    dat_per_cell_per_tstep = []
    for rec in state_recs:
        dat_per_cell = []
        for cell_rec in rec['interactions']:
            dat_per_cell.append(p2ds_to_numpy(cell_rec[dat_key]))
        dat_per_cell_per_tstep.append(np.array(dat_per_cell))
    return np.array(dat_per_cell_per_tstep)

def extract_scalars(state_key, dat_key, state_recs):
    dat_per_cell_per_tstep = []
    for rec in state_recs:
        dat_per_cell = []
        for cell_rec in rec['states']:
            dat_per_cell.append(np.array(cell_rec[state_key][dat_key]))
        dat_per_cell_per_tstep.append(np.array(dat_per_cell))
    return np.array(dat_per_cell_per_tstep)


poly_per_cell_per_tstep = extract_p2ds_from_cell_states('core', 'poly', state_recs)
centroids_per_cell_per_tstep = np.array(
    [[np.average(poly, axis=0) for poly in poly_per_cell] for poly_per_cell in
     poly_per_cell_per_tstep])
uivs_per_cell_per_tstep = extract_p2ds_from_cell_states('geom', 'unit_inward_vecs',
                                                        state_recs)
uovs_per_cell_per_tstep = -1 * uivs_per_cell_per_tstep
rac_acts_per_cell_per_tstep = extract_scalars('core', 'rac_acts', state_recs)
rac_act_arrows_per_cell_per_tstep = 50 * rac_acts_per_cell_per_tstep[:, :, :,
                                         np.newaxis] * uovs_per_cell_per_tstep
rho_acts_per_cell_per_tstep = extract_scalars('core', 'rho_acts', state_recs)
rho_act_arrows_per_cell_per_tstep = 50 * rho_acts_per_cell_per_tstep[:, :, :,
                                         np.newaxis] * uivs_per_cell_per_tstep

adhs_per_cell_per_tstep = 5*extract_p2ds_from_interactions('x_adhs', state_recs)



# output = None
# file_name = "../output/separated_pair_cil=30_cal=None_adh=10_coa=24_seed=7_rt.cbor"
# with open(file_name, mode='rb') as sf:
#     output = cbor2.load(sf)
# tsteps = [o[0] for o in output]
# frequency = tsteps[1] - tsteps[0]
# state_recs = [o[1] for o in output]
# interactions = [rec["interactions"] for rec in state_recs]
# x_adhs_0_4 = [inters[0]["x_adhs"][4] for inters in interactions]
# x_adhs_1_12 = [inters[1]["x_adhs"][12] for inters in interactions]
#
#
# plt.plot(tsteps, x_adhs_0_4, color="black", marker=".")
# plt.plot(tsteps, x_adhs_1_12, color="green", marker=".")
