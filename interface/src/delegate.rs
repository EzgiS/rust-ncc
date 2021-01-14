use crate::AppState;
use druid::{AppDelegate, DelegateCtx, Target, Command, Env, Handled, commands};
use std::sync::Arc;
use rust_ncc::world::hardio::load_binc_from_path;
use log::info;

pub struct Delegate;

impl AppDelegate<AppState> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        app: &mut AppState,
        _env: &Env,
    ) -> Handled {
        let open_command = cmd.get(commands::OPEN_FILE);
        if let Some(file_info) =  open_command {
            app.sim_history =
                Arc::new(load_binc_from_path(file_info.path()));
            app.frame = 0;
            return Handled::Yes;
        }
        Handled::No
    }
}
