import Sm64VisualEngine from "./scripts/sm64/Sm64Game.js";

function sleep(time) {
  return new Promise((resolve) => setTimeout(resolve, time));
}

async function iter_with_anim_frame(func, get_speed) {
    let done = false;
    let target_time = 0;
    const loop = (time) => {
        time *= 0.03; // milliseconds to frame count (33.333 ms -> 1)
        if (time >= target_time + 100.0) {
            // We are lagging 100 frames behind, probably due to coming back after inactivity,
            // so reset, with a small margin to avoid potential jitter later.
            target_time = time - 0.010;
        }
        while (time >= target_time + 1.0/get_speed()) {
            done = func();
            if (done) return;
            target_time = target_time + 1.0/get_speed();
        }
        requestAnimationFrame(loop);
    }
    requestAnimationFrame(loop);
    

    return new Promise((resolve) => {
        const checkEndCondition = () => {
            if (done) {
                resolve();
            } else {
                setTimeout(checkEndCondition, 100); // Polling every 100ms to check end conditions
            }
        };
        checkEndCondition();
    });
}

async function evaluate(canvas, seed, solution_bytes = []) {
    let [_, success] = playback(canvas, solution_bytes, seed)[1];
    return success;
}

async function playback(canvas, inputs, seed=NaN, controllable=false, min_speed=2, max_speed=10000) {
    let success = false;
    let engine;
    if (isNaN(seed)) {
        engine = await Sm64VisualEngine.create(canvas, 22);
    } else {
        engine = await Sm64VisualEngine.create(canvas, seed);
    }
    let speed = max_speed;
    function get_speed() {return speed}
    engine.set_audio_enabled(0);


    // Loop through all frames
    let i = 0;
    function step() {
        if (i >= inputs.length) return true;
        let pad = inputs[i];
        i+=1;

        // assert that the playback matches the seed
        let r_pad = engine.rng_pad(pad);
        if (!isNaN(seed) && !pad.equals(r_pad)) {
            throw new Error("Replay inputs do not match seed")
        }
        
        // playback
        engine.step_game(pad);

        // check if won
        let state = engine.get_game_state();
        if (state.hasWon()) {
            success = true;
            return true;
        }

        if (controllable) {
            // allow special inputs
            let c_pad = engine.get_controller_pad();
            if (c_pad.start()) {
                inputs.length = i;
                return true;
            }
            let cut_amount = 10 * min_speed * 30 // 5 seconds before
            if (inputs.length - cut_amount < i) {
                speed = min_speed;
            }
        }

        return false;
    }
    
    await iter_with_anim_frame(step, get_speed);
    engine.set_audio_enabled(1);
    return [engine, success];
}


async function record(canvas, seed=NaN, starting_bytes = []) {
    let [engine, success] = await playback(canvas, starting_bytes, seed, true);
    if (success) return [success, solution_bytes];

    let solution_bytes = Array.from(starting_bytes); // shallow copy, use the same pad objects but different array

    let speed = 1;
    function get_speed() {return speed}

    function step() {
        let c_pad = engine.get_controller_pad();
        // true controller pad, other pad can change via RNG

        let pad = c_pad;
        if (!isNaN(seed)) pad = engine.rng_pad(pad);
        
        engine.step_game(pad);
        solution_bytes.push(pad);

        let state = engine.get_game_state();
        // console.log(state.toString());

        // check if won
        if (state.hasWon()) {
            success = true;
            return true;
        }

        if (c_pad.u_jpad() ) {
            return true;
        }

        if (c_pad.d_jpad()) {
            speed = 10;
            engine.set_audio_enabled(0);
        } else {
            speed = 1;
            engine.set_audio_enabled(1);
        }
        return false;
    }
    
    await iter_with_anim_frame(step, get_speed);

    return [success, solution_bytes];

}

async function record_loop(canvas, seed) {
    var success = false;
    var starting_bytes = [];
    while (!success) {
        [success, starting_bytes] = await record(canvas, seed, starting_bytes);
    }
    return (starting_bytes);
}

export {record_loop};
