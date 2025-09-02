
use std::path::PathBuf;
use std::os::raw::{c_float, c_int, c_short};
use libloading::{Library, Symbol};
use rand::Rng;

use super::sm64_structs::{MarioState, GameInfo};

pub(crate) type Vec3f = [c_float; 3];


pub struct SM64Game {
    lib_path: PathBuf,
    step_game: libloading::os::unix::Symbol<unsafe extern "C" fn(c_int, c_int, c_int, u32)>,
    // reset: libloading::os::unix::Symbol<unsafe extern "C" fn()>,
    get_mario_state: libloading::os::unix::Symbol<unsafe extern "C" fn() -> *const MarioState>,
    get_game_info: libloading::os::unix::Symbol<unsafe extern "C" fn() -> GameInfo>,
    get_lakitu_pos: libloading::os::unix::Symbol<unsafe extern "C" fn() -> *const Vec3f>,
    get_lakitu_yaw: libloading::os::unix::Symbol<unsafe extern "C" fn() -> c_short>,
    end: libloading::os::unix::Symbol<unsafe extern "C" fn()>,
}
impl Drop for SM64Game {
    fn drop(&mut self) {
        // self.end();
        std::fs::remove_file(self.lib_path.clone()).unwrap();
    }
}
impl SM64Game {
    pub fn new(headless:bool) -> Result<Self, Box<dyn std::error::Error>> {
        let start_in_foreground = !headless;
        
        let mut path = std::env::current_dir()?;

        if headless {
            path.push("sm64_headless.us");
        } else {
            path.push("sm64.us");
        }

        // Make a copy of the library to avoid issues with loading it multiple times
        let mut rng = rand::rng();
        let rng_name = format!("sm64_{}.tmp.so", rng.random::<u64>());
        let mut lib_copy_path = std::env::current_dir()?;
        lib_copy_path.push("tmp_so");
        if !lib_copy_path.exists() {std::fs::create_dir_all(&lib_copy_path)?;}
        lib_copy_path.push(rng_name);
        std::fs::copy(&path, &lib_copy_path)?;
        let rng_path = &lib_copy_path;

        // Load the library
        let lib = unsafe { Library::new(rng_path)? };

        unsafe {
            let main_func: Symbol<unsafe extern "C" fn(c_int)> = lib.get(b"main_func")?;
            main_func(if start_in_foreground { 1 } else { 0 });
            Ok(SM64Game {
                step_game: lib.get::<unsafe extern "C" fn(c_int, c_int, c_int, u32)>(b"step_game")?.into_raw(),
                // reset: lib.get::<unsafe extern "C" fn()>(b"reset")?.into_raw(),
                get_mario_state: lib.get::<unsafe extern "C" fn() -> *const MarioState>(b"get_mario_state\0")?.into_raw(),
                get_game_info: lib.get::<unsafe extern "C" fn() -> GameInfo>(b"get_game_info\0")?.into_raw(),
                get_lakitu_pos: lib.get::<unsafe extern "C" fn() -> *const Vec3f>(b"get_lakitu_pos")?.into_raw(),
                get_lakitu_yaw: lib.get::<unsafe extern "C" fn() -> c_short>(b"get_lakitu_yaw")?.into_raw(),
                end: lib.get::<unsafe extern "C" fn()>(b"end")?.into_raw(),
                lib_path: rng_path.to_path_buf(),
            })
        }
    }

    pub fn step_game(&self, num_steps: i8, stick_x: i8, stick_y: i8, button: u16) {
        unsafe {
            (self.step_game)(num_steps.into(), stick_x.into(), stick_y.into(), button.into());
        }
    }

    // pub fn reset(&self) {
    //     unsafe { (self.reset)() }
    // }

    pub fn end(&self) {
        unsafe { (self.end)() }
    }

    pub fn get_mario_state(&self) -> MarioState {
        unsafe { *(self.get_mario_state)() }
    }

    pub fn get_game_info(&self) -> GameInfo {
        unsafe { (self.get_game_info)() }
    }

    pub fn get_lakitu_pos(&self) -> [f32; 3] {
        unsafe { *(self.get_lakitu_pos)() }
    }

    pub fn get_lakitu_yaw(&self) -> i16 {
        unsafe { (self.get_lakitu_yaw)() }
    }
}
