use wasmtime::*;

use std::convert::TryInto;


const A_BUTTON: u16 = 0x8000;
const B_BUTTON: u16 = 0x4000;
const L_TRIG: u16 = 0x0020;
const R_TRIG: u16 = 0x0010;
const Z_TRIG: u16 = 0x2000;
const START_BUTTON: u16 = 0x1000;
const U_JPAD: u16 = 0x0800;
const L_JPAD: u16 = 0x0200;
const R_JPAD: u16 = 0x0100;
const D_JPAD: u16 = 0x0400;
const U_CBUTTONS: u16 = 0x0008;
const L_CBUTTONS: u16 = 0x0002;
const R_CBUTTONS: u16 = 0x0001;
const D_CBUTTONS: u16 = 0x0004;

#[derive(Debug)]
struct GameState {
    num_stars: i32,
    pos: [f32; 3],
    vel: [f32; 3],
    lakitu_pos: [f32; 3],
    lakitu_yaw: i32,
    in_credits: i32,
    course_num: i32,
    act_num: i32,
    area_index: i32,
}

impl GameState {
    fn new(data: &[u8]) -> GameState {
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

    fn has_won(&self) -> bool {
        self.num_stars > 0
    }

    fn to_string(&self) -> String {
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




fn main() -> wasmtime::Result<()> {
    let engine = Engine::default();
    let mut store: Store<u32> = Store::new(&engine, 4);
    // let memory = Memory::new(&mut store, MemoryType::new(10000, None))?;

    // Modules can be compiled through either the text or binary format
    // let wat = include_bytes!("../../WASM/sm64_headless.us.wasm");
    // let module = Module::new(&engine, wat)?;
    let module = Module::from_file(&engine, "../WASM/sm64_headless.us.wasm")?;
    let mut linker = Linker::new(&engine);
    linker.define_unknown_imports_as_traps(&module)?;
    let instance = linker.instantiate(&mut store, &module)?;

    let memory = instance.get_memory(&mut store, "memory").unwrap();

    let main_func = instance.get_typed_func::<(), ()>(&mut store, "main_func")?;
    let step_game = instance.get_typed_func::<(u32, i32, i32), ()>(&mut store, "step_game")?;
    let get_game_state = instance.get_typed_func::<(), i32>(&mut store, "get_game_state")?;
    // let main_func = instance.get_typed_func::<(), ()>(&mut store, "main_func")?;
    // let main_func = instance.get_typed_func::<(), ()>(&mut store, "main_func")?;
    // let main_func = instance.get_typed_func::<(), ()>(&mut store, "main_func")?;

    // And finally we can call the wasm!
    main_func.call(&mut store, ())?;

    let mut i = 0;
    while i < 2000 {

        let mut button: u16 = 0;
        let mut stickx: i8 = 0;
        let mut sticky: i8 = 0;

        if (150 < i && i < 160) || (200 < i && i < 300) {
            button = START_BUTTON;
        } else {
            button = 0;
        }

        if i > 300 {
            sticky = 80;
        }

        if i % 2 == 0 {
            button = A_BUTTON;
        }


        step_game.call(&mut store, (button.into(), stickx.into(), sticky.into()))?;


        let pointer = get_game_state.call(&mut store, ())?;

        let mut buffer: [u8; 60] = [0; 60];
        memory.read(&mut store, pointer.try_into()?, &mut buffer)?;

        let value = GameState::new(&buffer);
        println!("Buffer contents: {:?}", &buffer);
        println!("{}\n", value.to_string());

        // if value.pos[0] != 0f32 {
        //     println!("{} {} {}", value.pos[0], value.pos[1], value.pos[2] );
        // }

        i += 1;
    }

    Ok(())
}