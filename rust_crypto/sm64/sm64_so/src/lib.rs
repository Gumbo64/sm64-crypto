
mod util;
mod replay;
mod constants;
mod sm64_structs;
mod game;
// ---------- selective re‑exports ----------
pub use util::{remove_tmp_so_files, buttons_to_int, StatefulInputGenerator, eval_metric};
pub use replay::{Replay};
pub use constants::{BUTTONS};
pub use sm64_structs::{MarioState, GameInfo};
pub use game::SM64Game;
// ------------

