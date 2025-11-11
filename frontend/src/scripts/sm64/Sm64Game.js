import { isRomCached, getFile } from "../fileUpload.js"
import SM64_LIB from "./pkg/sm64.us.js"
import { DEFAULT_CONFIG } from "./randomConfig.js";

async function instantiateWasmSM64(info, func) {
    const wasmBuffer = await getFile("sm64.us.wasm");
    const instance = await WebAssembly.instantiate(wasmBuffer, info);
    func(instance["instance"], instance["module"]);
}

const littleEndian = true;

class GamepadButtons {
    static A_BUTTON = 0x8000;
    static B_BUTTON = 0x4000;
    static L_TRIG = 0x0020;
    static R_TRIG = 0x0010;
    static Z_TRIG = 0x2000;
    static START_BUTTON = 0x1000;
    static U_JPAD = 0x0800;
    static L_JPAD = 0x0200;
    static R_JPAD = 0x0100;
    static D_JPAD = 0x0400;
    static U_CBUTTONS = 0x0008;
    static L_CBUTTONS = 0x0002;
    static R_CBUTTONS = 0x0001;
    static D_CBUTTONS = 0x0004;
}

class GamePad {
    constructor(button, stick_x, stick_y) {
        this.button = button;
        this.stick_x = stick_x;
        this.stick_y = stick_y;
    }
    equals(pad) {
        if (!(pad instanceof GamePad)) {
            return false;
        }
        return this.button == pad.button && this.stick_x == pad.stick_x && this.stick_y == pad.stick_y;
    }
    static from_pointer(pointer, engine) {
        // typedef struct {
        // u16     button;
        // s8      stick_x;		/* -80 <= stick_x <= 80 */
        // s8      stick_y;		/* -80 <= stick_y <= 80 */
        // u8	errnum;
        // } OSContPad;
        const view = new DataView(engine.mem_slice(pointer, 4));
        
        return new GamePad(
            view.getUint16(0, littleEndian),
            view.getInt8(2),
            view.getInt8(3),
        );
    }
    is_pressed(button_mask) {
        return this.button & button_mask;
    }
    enable_button(button_mask) {
        this.button |= button_mask;
    }
    disable_button(button_mask) {
        this.button &= ~button_mask;
    }
    clone() {
        return new GamePad(
            this.button,
            this.stick_x,
            this.stick_y,
        );
    }
}

class GameState {
    constructor(pointer, engine) {
// struct GameState {
//     s32 numStars;
//     Vec3f pos;
//     Vec3f vel;
//     Vec3f lakituPos;
//     s32 lakituYaw;
//     s32 inCredits;
//     s32 courseNum;
//     s32 actNum;
//     s32 areaIndex;
// };

        let size = 4 * (6 + 3*3);
        const view = new DataView(engine.mem_slice(pointer, size));

        let i = 0;
        // it should be 2 bytes but idk this is how it works. prolly some WASM
        this.numStars = view.getUint16(i, littleEndian); i += 4;
        
        this.pos = [];
        this.pos.push(view.getFloat32(i, littleEndian)); i += 4;
        this.pos.push(view.getFloat32(i, littleEndian)); i += 4;
        this.pos.push(view.getFloat32(i, littleEndian)); i += 4;

        this.vel = [];
        this.vel.push(view.getFloat32(i, littleEndian)); i += 4;
        this.vel.push(view.getFloat32(i, littleEndian)); i += 4;
        this.vel.push(view.getFloat32(i, littleEndian)); i += 4;

        
        this.lakituPos = [];
        this.lakituPos.push(view.getFloat32(i, littleEndian)); i += 4;
        this.lakituPos.push(view.getFloat32(i, littleEndian)); i += 4;
        this.lakituPos.push(view.getFloat32(i, littleEndian)); i += 4;

        this.lakituYaw = view.getUint16(i, littleEndian); i += 4;

        this.inCredits = view.getUint8(i); i += 4;

        this.courseNum = view.getUint16(i, littleEndian); i += 4;

        this.actNum = view.getUint16(i, littleEndian); i += 4;

        this.areaIndex = view.getUint16(i, littleEndian);
    }
    hasWon() {
        return this.numStars > 0;
    }
    toString() {
        return `GameState {
            numStars: ${this.numStars},
            position: (${this.pos.join(", ")}),
            velocity: (${this.vel.join(", ")}),
            lakituPosition: (${this.lakituPos.join(", ")}),
            lakituYaw: ${this.lakituYaw},
            inCredits: ${this.inCredits},
            courseNum: ${this.courseNum},
            actNum: ${this.actNum},
            areaIndex: ${this.areaIndex}
        }`;
    }
}


class Sm64VisualEngine {
    constructor(game, memory, rng_config) {
        this.game = game;
        this.memory = memory
        this.rng_config = rng_config;
    }
    static async create(canvas, seed, rng_config=DEFAULT_CONFIG) {
        let memory = new WebAssembly.Memory({
            initial: 10000,
            maximum: 10000,
        });
        let game = await SM64_LIB({
            "instantiateWasm": instantiateWasmSM64,
            "canvas": canvas,
            "wasmMemory": memory
        });

        if (rng_config != null) {
            game._rng_init(
                seed, 
                rng_config["max_random_action"], 
                rng_config["max_window_length"], 
                rng_config["A_prob"], 
                rng_config["B_prob"], 
                rng_config["Z_prob"]
            )
        }
        game._main_func();

        return new Sm64VisualEngine(game, memory, rng_config);
    }

    mem_slice(pointer, length) {
        return this.memory.buffer.slice(pointer, pointer + length);
    }

    step_game(pad) {
        return this.game._step_game(pad.button, pad.stick_x, pad.stick_y);
    }

    rng_pad(pad) {
        let pointer = this.game._rng_pad(pad.button, pad.stick_x, pad.stick_y);
        return GamePad.from_pointer(pointer, this);
    }

    // get_mario_state() {
    //     let pointer = this.game._get_mario_state();
    //     const view = new DataView(this.mem_slice(pointer, 0x3C + 12));

    //     let i = 0x3C;
        
    //     let pos = [];
    //     pos.push(view.getFloat32(i, littleEndian)); i += 4;
    //     pos.push(view.getFloat32(i, littleEndian)); i += 4;
    //     pos.push(view.getFloat32(i, littleEndian)); i += 4;
    //     return pos;
    // }

    get_controller_pad() {
        let pointer = this.game._get_controller_pad();
        return GamePad.from_pointer(pointer, this);
    }
    get_game_state() {
        let pointer = this.game._get_game_state();
        return new GameState(pointer, this);
    }
    set_audio_enabled(truth) {
        return this.game._set_audio_enabled(truth);
    }

}

export {Sm64VisualEngine, GamePad, GamepadButtons};
