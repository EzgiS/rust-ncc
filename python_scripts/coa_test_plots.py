import matplotlib.pyplot as plt
import json
import numpy as np
import json
import cbor2

output = None

file_name = "../output/four_cell_cil=60_cal=None_adh=None_coa=1_seed=7_rf.cbor"

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


def v2ds_to_numpy(v2ds):
    vs = []
    for v2d in v2ds:
        vs.append([v2d['x'], v2d['y']])
    return np.array(vs)


def v2d_to_numpy(v2d):
    v = [v2d['x'], v2d['y']]
    return np.array(v)


def extract_v2ds_from_cell_states(state_key, dat_key, state_recs):
    dat_per_cell_per_tstep = []
    for rec in state_recs:
        dat_per_cell = []
        for cell_rec in rec['states']:
            dat_per_cell.append(v2ds_to_numpy(cell_rec[state_key][dat_key]))
        dat_per_cell_per_tstep.append(np.array(dat_per_cell))
    return np.array(dat_per_cell_per_tstep)


def extract_v2ds_from_interactions(dat_key, state_recs):
    dat_per_cell_per_tstep = []
    for rec in state_recs:
        dat_per_cell = []
        for cell_rec in rec['interactions']:
            dat_per_cell.append(v2ds_to_numpy(cell_rec[dat_key]))
        dat_per_cell_per_tstep.append(np.array(dat_per_cell))
    return np.array(dat_per_cell_per_tstep)


# def extract_vector_to_close_points_from_interactions(state_recs):
#     dat_per_cell_per_tstep = []
#     for rec in state_recs:
#         dat_per_cell = []
#         for cell_rec in rec['interactions']:
#             x_close_points = cell_rec['x_close_points']
#             dat_per_vertex = []
#             for vertex in x_close_points:
#                 if vertex['empty']:
#                     vertex.append(np.array([np.nan, np.nan]))
#                 else:
#                     vector_to = vertex['vector_to']
#                     dat_per_vertex.append(v2d_to_numpy(vector_to))
#             dat_per_cell.append(np.array(dat_per_vertex))
#         dat_per_cell_per_tstep.append(np.array(dat_per_cell))
#     return np.array(dat_per_cell_per_tstep)


def extract_scalars(state_key, dat_key, state_recs):
    dat_per_cell_per_tstep = []
    for rec in state_recs:
        dat_per_cell = []
        for cell_rec in rec['states']:
            dat_per_cell.append(np.array(cell_rec[state_key][dat_key]))
        dat_per_cell_per_tstep.append(np.array(dat_per_cell))
    return np.array(dat_per_cell_per_tstep)


def extract_scalars_from_interactions(dat_key, state_recs):
    dat_per_cell_per_tstep = []
    for rec in state_recs:
        dat_per_cell = []
        for cell_rec in rec['interactions']:
            dat_per_cell.append(np.array(cell_rec[dat_key]))
        dat_per_cell_per_tstep.append(np.array(dat_per_cell))
    return np.array(dat_per_cell_per_tstep)


poly_per_cell_per_tstep = extract_v2ds_from_cell_states('core', 'poly',
                                                        state_recs)
centroids_per_cell_per_tstep = np.array(
    [[np.average(poly, axis=0) for poly in poly_per_cell] for poly_per_cell in
     poly_per_cell_per_tstep])
rac_acts_per_cell_per_tstep = extract_scalars('core', 'rac_acts', state_recs)
rho_acts_per_cell_per_tstep = extract_scalars('core', 'rho_acts', state_recs)

adhs_per_cell_per_tstep = extract_v2ds_from_interactions('x_adhs', state_recs)
rgtp_forces_per_cell_per_tstep = extract_v2ds_from_cell_states("mech",
                                                               "rgtp_forces",
                                                               state_recs)
edge_forces_per_cell_per_tstep = extract_v2ds_from_cell_states("mech",
                                                               "edge_forces",
                                                               state_recs)
cyto_forces_per_cell_per_tstep = extract_v2ds_from_cell_states("mech",
                                                               "cyto_forces",
                                                               state_recs)
sum_non_adh_forces_per_cell_per_tstep = extract_v2ds_from_cell_states("mech",
                                                                      "sum_forces",
                                                                      state_recs)
# vector_to_close_point_per_cell_per_tstep = extract_vector_to_close_points_from_interactions(state_recs)
# magnitudes_of_close_point_vectors = np.linalg.norm(vector_to_close_point_per_cell_per_tstep, axis=3)
# cils_per_cell_per_tstep = extract_scalars_from_interactions('x_cils', state_recs)

coa_per_cell_per_tstep = extract_scalars_from_interactions('x_coas', state_recs)
rac_krgtp_per_cell_per_tstep = extract_scalars('chem', 'kgtps_rac', state_recs)


rac_cell_0 = rac_acts_per_cell_per_tstep[:, 0, :]
rac_cell_1 = rac_acts_per_cell_per_tstep[:, 1, :]
rac_cell_2 = rac_acts_per_cell_per_tstep[:, 2, :]
rac_cell_3 = rac_acts_per_cell_per_tstep[:, 3, :]

rac_krgtp_cell_0 = coa_per_cell_per_tstep[:, 0, :]
rac_krgtp_cell_1 = coa_per_cell_per_tstep[:, 1, :]
rac_krgtp_cell_2 = coa_per_cell_per_tstep[:, 2, :]
rac_krgtp_cell_3 = coa_per_cell_per_tstep[:, 3, :]

coa_cell_0 = coa_per_cell_per_tstep[:, 0, :]
coa_cell_1 = coa_per_cell_per_tstep[:, 1, :]
coa_cell_2 = coa_per_cell_per_tstep[:, 2, :]
coa_cell_3 = coa_per_cell_per_tstep[:, 3, :]

plt.plot(tsteps, rac_cell_0, color="black", marker=".")
plt.title('Rac acts on Cell 0')
plt.show()

plt.plot(tsteps, rac_cell_1, color="black", marker=".")
plt.title('Rac acts on Cell 1')
plt.show()

plt.plot(tsteps, rac_cell_2, color="black", marker=".")
plt.title('Rac acts on Cell 2')
plt.show()

plt.plot(tsteps, rac_cell_3, color="black", marker=".")
plt.title('Rac acts on Cell 3')
plt.show()

plt.plot(tsteps, rac_krgtp_cell_0, color="blue", marker=".")
plt.title('rac_krgtp on Cell 0')
plt.show()

plt.plot(tsteps, rac_krgtp_cell_1, color="blue", marker=".")
plt.title('rac_krgtp on Cell 1')
plt.show()

plt.plot(tsteps, rac_krgtp_cell_2, color="blue", marker=".")
plt.title('rac_krgtp on Cell 2')
plt.show()

plt.plot(tsteps, rac_krgtp_cell_3, color="blue", marker=".")
plt.title('rac_krgtp on Cell 3')
plt.show()

plt.plot(tsteps, coa_cell_0, color="green", marker=".")
plt.title('coa on Cell 0')
plt.show()

plt.plot(tsteps, coa_cell_1, color="green", marker=".")
plt.title('coa on Cell 1')
plt.show()

plt.plot(tsteps, coa_cell_2, color="green", marker=".")
plt.title('coa on Cell 2')
plt.show()

plt.plot(tsteps, coa_cell_3, color="green", marker=".")
plt.title('coa on Cell 3')
plt.show()