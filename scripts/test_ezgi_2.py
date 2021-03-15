from sim_data import SimulationData
from sim_data import SharedSimData
from paint_opts import *
from utils import *
import os
import subprocess
import toml

run_experiments = True
root_dir = os.getcwd()
exp_tomls = ["one_cell", "one_cell_no_coa"] # ["one_cell", "four_cell"]
sim_dats = []
vec_ani_opts = []
out_dir = None
for exp_toml in exp_tomls:
    exec_mode = "release"
    exec_path = os.path.join(root_dir, "target", exec_mode, "executor")
    if run_experiments:
        build_out = subprocess.run(["cargo", "build"] + make_exec_mode_arg(
            exec_mode) + ["-p", "executor"])
        run_out = subprocess.run([exec_path] +
                                 ["-c", "./cfg.toml"] +
                                 ["-e"] + exp_tomls)
        print(run_out)

    exp_dict = toml.load(
        os.path.join(root_dir, "experiments", "{}.toml".format(exp_toml))
    )
    file_names = determine_file_names(exp_toml, exp_dict)
    out_dir = os.path.join(root_dir, "output")
    for (ix, file_name) in enumerate(file_names):
        rust_dat = SimulationData()
        rust_dat.load_rust_dat(out_dir, file_name)
        if ix == 0:
            vec_ani_opts = get_vec_ani_opts(exp_dict)
        sim_dats.append(rust_dat)

print(vec_ani_opts)
comp_dat = SharedSimData(out_dir, sim_dats, ["-", ":"], "coa_no_coa")
comp_dat.animate(vec_ani_opts)
