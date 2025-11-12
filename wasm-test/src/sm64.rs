use wasmtime::*;

use std::convert::TryInto;

use crate::sm64_util;



#[derive(Clone, Debug)]
pub struct GamePad {
    button: u16,
    stick_x: i8,
    stick_y: i8,
}

impl GamePad {
    pub fn new(button: u16, stick_x: i8, stick_y: i8) -> Self {
        GamePad { button, stick_x, stick_y }
    }

    pub fn equals(&self, pad: &GamePad) -> bool {
        self.button == pad.button && self.stick_x == pad.stick_x && self.stick_y == pad.stick_y
    }

    pub fn from_bytes(data: &[u8]) -> Self {
        let button = u16::from_le_bytes(data[0..1].try_into().unwrap());
        let stick_x = data[2] as i8;
        let stick_y = data[3] as i8;
        GamePad::new(button, stick_x, stick_y)
    }

    pub fn is_pressed(&self, button_mask: u16) -> bool {
        (self.button & button_mask) != 0
    }

    pub fn enable_button(&mut self, button_mask: u16) {
        self.button |= button_mask;
    }

    pub fn disable_button(&mut self, button_mask: u16) {
        self.button &= !button_mask;
    }
}


#[derive(Debug)]
pub struct GameState {
    pub num_stars: i32,
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub lakitu_pos: [f32; 3],
    pub lakitu_yaw: i32,
    pub in_credits: i32,
    pub course_num: i32,
    pub act_num: i32,
    pub area_index: i32,
}

impl GameState {
    pub fn new(data: &[u8]) -> GameState {
        let mut offset = 0;

        let num_stars = i32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
        offset += 4;

        let pos = [
            f32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()),
            f32::from_le_bytes(data[offset + 4..offset + 8].try_into().unwrap()),
            f32::from_le_bytes(data[offset + 8..offset + 12].try_into().unwrap()),
        ];
        offset += 12;

        let vel = [
            f32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()),
            f32::from_le_bytes(data[offset + 4..offset + 8].try_into().unwrap()),
            f32::from_le_bytes(data[offset + 8..offset + 12].try_into().unwrap()),
        ];
        offset += 12;

        let lakitu_pos = [
            f32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()),
            f32::from_le_bytes(data[offset + 4..offset + 8].try_into().unwrap()),
            f32::from_le_bytes(data[offset + 8..offset + 12].try_into().unwrap()),
        ];
        offset += 12;

        let lakitu_yaw = i32::from_le_bytes(data[offset..offset+4].try_into().unwrap());
        offset += 4;

        let in_credits = i32::from_le_bytes(data[offset..offset+4].try_into().unwrap());
        offset += 4; 

        let course_num = i32::from_le_bytes(data[offset..offset+4].try_into().unwrap());
        offset += 4;

        let act_num = i32::from_le_bytes(data[offset..offset+4].try_into().unwrap());
        offset += 4;

        let area_index = i32::from_le_bytes(data[offset..offset+4].try_into().unwrap());

        // Construct GameState
        GameState {
            num_stars,
            pos,
            vel,
            lakitu_pos,
            lakitu_yaw,
            in_credits,
            course_num,
            act_num,
            area_index,
        }
    }

    pub fn has_won(&self) -> bool {
        self.num_stars > 0
    }

    pub fn to_string(&self) -> String {
        format!(
            "GameState {{\n\
                numStars: {},\n\
                position: ({}, {}, {}),\n\
                velocity: ({}, {}, {}),\n\
                lakituPosition: ({}, {}, {}),\n\
                lakituYaw: {},\n\
                inCredits: {},\n\
                courseNum: {},\n\
                actNum: {},\n\
                areaIndex: {}\n\
            }}",
            self.num_stars,
            self.pos[0], self.pos[1], self.pos[2],
            self.vel[0], self.vel[1], self.vel[2],
            self.lakitu_pos[0], self.lakitu_pos[1], self.lakitu_pos[2],
            self.lakitu_yaw,
            self.in_credits,
            self.course_num,
            self.act_num,
            self.area_index,
        )
    }
}


pub struct SM64Game {
    store: Store<u32>,
    instance: Instance,
    memory: Memory,
}

impl SM64Game {
    pub fn new(module_path: &str) -> Result<Self, Error> {
        let engine = Engine::default();
        let mut store: Store<u32> = Store::new(&engine, 4);
        let module = Module::from_file(&engine, module_path)?;
        
        let mut linker = Linker::new(&engine);
        linker.define_unknown_imports_as_traps(&module)?;
        
        let instance = linker.instantiate(&mut store, &module)?;
        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let main_func = instance.get_typed_func::<(), ()>(&mut store, "main_func")?;
        main_func.call(&mut store, ())?;

        Ok(SM64Game {
            store,
            instance,
            memory,
        })
    }


    pub fn step_game(&mut self, pad: GamePad) -> Result<(), Error> {
        let step_game = self.instance.get_typed_func::<(u32, i32, i32), ()>(&mut self.store, "step_game")?;
        step_game.call(&mut self.store, (pad.button.into(), pad.stick_x.into(), pad.stick_y.into()))?;
        Ok(())
    }

    pub fn get_game_state(&mut self) -> Result<GameState, Error> {
        let get_game_state = self.instance.get_typed_func::<(), i32>(&mut self.store, "get_game_state")?;
        let pointer = get_game_state.call(&mut self.store, ())?;
        let mut buffer: [u8; 60] = [0; 60];
        self.memory.read(&mut self.store, pointer.try_into()?, &mut buffer)?;
        let state = GameState::new(&buffer);
        Ok(state)
    }

    pub fn rng_init(&mut self, seed: u32) -> Result<(), Error> {
        let rng_init = self.instance.get_typed_func::<(u32, u32, u32, f32, f32, f32), ()>(&mut self.store, "rng_init")?;
        rng_init.call(&mut self.store, (
            seed, sm64_util::MAX_RANDOM_ACTION, sm64_util::MAX_WINDOW_LENGTH,
            sm64_util::A_PROB, sm64_util:: B_PROB, sm64_util::Z_PROB)
        )?;
        Ok(())
    }
    pub fn rng_pad(&mut self, pad: GamePad) -> Result<GamePad, Error> {
        let rng_pad = self.instance.get_typed_func::<(u32, i32, i32), i32>(&mut self.store, "rng_pad")?;
        let pointer = rng_pad.call(&mut self.store, (pad.button.into(), pad.stick_x.into(), pad.stick_y.into()))?;
        let mut buffer: [u8; 4] = [0; 4];
        self.memory.read(&mut self.store, pointer.try_into()?, &mut buffer)?;
        let pad = GamePad::from_bytes(&buffer);
        Ok(pad)
    }
}

