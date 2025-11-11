use wasmtime::*;

use std::convert::TryInto;

#[derive(Debug)]
struct GameState {
    num_stars: u16,
    pos: [f32; 3],
    vel: [f32; 3],
    lakitu_pos: [f32; 3],
    lakitu_yaw: u16,
    in_credits: u8,
    course_num: u16,
    act_num: u16,
    area_index: u16,
}

impl GameState {
    fn new(data: &[u8]) -> GameState {
        let mut offset = 0;

        let num_stars = u16::from_le_bytes(data[offset..offset+2].try_into().unwrap());
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

        let lakitu_yaw = u16::from_le_bytes(data[offset..offset + 2].try_into().unwrap());
        offset += 4;

        let in_credits = data[offset];
        offset += 4; 

        let course_num = u16::from_le_bytes(data[offset..offset + 2].try_into().unwrap());
        offset += 4;

        // Read act_num (2 bytes)
        let act_num = u16::from_le_bytes(data[offset..offset + 2].try_into().unwrap());
        offset += 4;

        // Read area_index (2 bytes)
        let area_index = u16::from_le_bytes(data[offset..offset + 2].try_into().unwrap());

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

    // Modules can be compiled through either the text or binary format
    // let wat = include_bytes!("../../WASM/sm64_headless.us.wasm");
    // let module = Module::new(&engine, wat)?;
    let module = Module::from_file(&engine, "../WASM/sm64_headless.us.wasm")?;

    // Host functionality can be arbitrary Rust functions and is provided
    // to guests through a `Linker`.
    let mut linker = Linker::new(&engine);
    linker.define_unknown_imports_as_traps(&module)?;

    // All wasm objects operate within the context of a "store". Each
    // `Store` has a type parameter to store host-specific data, which in
    // this case we're using `4` for.
    let mut store: Store<u32> = Store::new(&engine, 4);

    let memory = Memory::new(&mut store, MemoryType::new(10000, None))?;
    // Instantiation of a module requires specifying its imports and then
    // afterwards we can fetch exports by name, as well as asserting the
    // type signature of the function with `get_typed_func`.
    let instance = linker.instantiate(&mut store, &module)?;

    let main_func = instance.get_typed_func::<(), ()>(&mut store, "main_func")?;
    let step_game = instance.get_typed_func::<(u32, i32, i32), ()>(&mut store, "step_game")?;
    let get_mario_state = instance.get_typed_func::<(), i32>(&mut store, "get_mario_state")?;
    // let main_func = instance.get_typed_func::<(), ()>(&mut store, "main_func")?;
    // let main_func = instance.get_typed_func::<(), ()>(&mut store, "main_func")?;
    // let main_func = instance.get_typed_func::<(), ()>(&mut store, "main_func")?;

    // And finally we can call the wasm!
    main_func.call(&mut store, ())?;

    // let mut i = 0;
    // while i < 2000 {

    //     let button;
    //     let stickx;
    //     let sticky;

    //     if (150 < i && i < 160) || (200 < i && i < 300) {
    //         button = 16384;
    //     } else {
    //         button = 0;
    //     }

    //     if (i > 300) {
    //         sticky = 80;
    //     }


    //     step_game.call(&mut store, (button, 0, 0))?;


    //     let pointer = get_mario_state.call(&mut store, ())?;

    //     let mut buffer: [u8; 60] = [0; 60];
    //     memory.read(&mut store, pointer.try_into()?, &mut buffer)?;

    //     let value = GameState::new(&buffer);
    //     println!("Buffer contents: {:?}", &buffer);
    //     if value.pos[0] != 0f32 {
    //         println!("{} {} {}", value.pos[0], value.pos[1], value.pos[2] );
    //     }

    //     i += 1;
    // }

    Ok(())
}