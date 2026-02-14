use parking_lot::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunMode {
    Gui,
    Cli,
}

pub static RUN_MODE: Mutex<RunMode> = Mutex::new(RunMode::Gui);
