import matplotlib.pyplot as plt
import json
import numpy as np
import json
import cbor2

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
    return int(np.floor(tstep / frequency))


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


poly_per_cell_per_tstep = extract_p2ds_from_cell_states('core', 'poly',
                                                        state_recs)
centroids_per_cell_per_tstep = np.array(
    [[np.average(poly, axis=0) for poly in poly_per_cell] for poly_per_cell in
     poly_per_cell_per_tstep])
rac_acts_per_cell_per_tstep = extract_scalars('core', 'rac_acts', state_recs)
rho_acts_per_cell_per_tstep = extract_scalars('core', 'rho_acts', state_recs)

adhs_per_cell_per_tstep = extract_p2ds_from_interactions('x_adhs', state_recs)
rgtp_forces_per_cell_per_tstep = extract_p2ds_from_cell_states("mech",
                                                               "rgtp_forces",
                                                               state_recs)
edge_forces_per_cell_per_tstep = extract_p2ds_from_cell_states("mech",
                                                               "edge_forces",
                                                               state_recs)
cyto_forces_per_cell_per_tstep = extract_p2ds_from_cell_states("mech",
                                                               "cyto_forces",
                                                               state_recs)
sum_non_adh_forces_per_cell_per_tstep = extract_p2ds_from_cell_states("mech",
                                                                      "sum_forces",
                                                                      state_recs)
sum_non_adhs = np.sum(sum_non_adh_forces_per_cell_per_tstep, axis=2)

sum_adhs = np.sum(adhs_per_cell_per_tstep, axis=2)
sum_adh_mags = np.sum(sum_adhs, axis=2)

sum_rgtps = np.sum(rgtp_forces_per_cell_per_tstep, axis=2)
sum_edgefs_per_cell_per_tstep = np.sum(edge_forces_per_cell_per_tstep, axis=2)
sum_cytofs_per_cell_per_tstep = np.sum(cyto_forces_per_cell_per_tstep, axis=2)

total_non_adhs = sum_rgtps + \
                 sum_edgefs_per_cell_per_tstep + \
                 sum_cytofs_per_cell_per_tstep
total_non_adh_mags = np.linalg.norm(
    total_non_adhs, axis=2)

total_non_adh_uvs = \
    total_non_adhs / \
    total_non_adh_mags[:, :, np.newaxis]

sum_f_mags = np.linalg.norm(sum_non_adhs, axis=2)
sum_f_uvs = sum_non_adhs / sum_f_mags[:, :, np.newaxis]

adh_proj_on_rgtp_dirn = \
    np.array([[np.dot(sum_adhs[t_ix][c_ix], sum_f_uvs[t_ix, c_ix])
               for c_ix in range(sum_f_uvs.shape[1])]
              for t_ix in range(sum_f_uvs.shape[0])])

adh_plus_rgtp = adh_proj_on_rgtp_dirn + sum_f_mags
plt.plot(adh_proj_on_rgtp_dirn[:, 0], color="red")
plt.plot(sum_f_mags[:, 0], color="blue")
plt.plot(adh_plus_rgtp[:, 0], color="green")

# plt.plot(sum_non_adhs[:, 0, 0], color="black")
# plt.plot(total_non_adhs[:, 0, 0], color="black")

