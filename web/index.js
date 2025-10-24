function sleep(time) {
  return new Promise((resolve) => setTimeout(resolve, time));
}

const MAX_NAME_LENGTH = 64;
const MAX_SOLUTION_TIME = 600; // 600 seconds = 10 minutes
const MAX_WINDOW_LENGTH = 100;
const MAX_RANDOM_ACTION = 5;

const MAX_SOLUTION_BYTES = MAX_SOLUTION_TIME * 30 * 4; // seconds * fps * (bytes per frame) 

function make_config(max_name_length, max_solution_bytes, max_window_length, max_random_action) {
    return {
        "max_name_length": max_name_length,
        "max_solution_bytes": max_solution_bytes,
        "max_window_length": max_window_length,
        "max_random_action": max_random_action,
    }
}

const DEFAULT_CONFIG = make_config(MAX_NAME_LENGTH, MAX_SOLUTION_BYTES, MAX_WINDOW_LENGTH, MAX_RANDOM_ACTION)

function intToUint8Array(value) {
    // Ensure the value is an integer
    if (!Number.isInteger(value)) {
        throw new Error("Value must be an integer.");
    }
    
    // Create a Uint8Array of length 4 (for 32-bit integers)
    const arr = new Uint8Array(4);
    
    // Use DataView to set the integer at the appropriate byte position
    const view = new DataView(arr.buffer);
    view.setUint32(0, value, true); // true for little-endian format
    
    return arr;
}
function create_info_file(game, info_filename, seed, record_mode, config) {
    var stream = game.FS.open(info_filename, 'w+');

    game.FS.write(stream, intToUint8Array(seed), 0, 4);
    game.FS.write(stream, intToUint8Array(record_mode), 0, 4);

    game.FS.write(stream, intToUint8Array(config["max_solution_bytes"]), 0, 4);
    game.FS.write(stream, intToUint8Array(config["max_window_length"]), 0, 4);
    game.FS.write(stream, intToUint8Array(config["max_random_action"]), 0, 4);

    game.FS.close(stream);
    return info_filename;
}

async function evaluate(seed, filename, solution_bytes = [], headless = true) {
    var statusCode = NaN;
    var game;
    if (headless) {
        game = await SM64_HEADLESS({
            "canvas": document.querySelector("#canvas"), 
            "onExit": (e) => {
                statusCode = e;
            }
        });
    } else {
        game = await SM64({
            "canvas": document.querySelector("#canvas"), 
            "onExit": (e) => {
                statusCode = e;
            }
        });
    }
    
    if (solution_bytes) {
        var stream = game.FS.open(filename, 'w+');
        game.FS.write(stream, solution_bytes, 0, solution_bytes.length, 0);
        game.FS.close(stream);
    }

    info_filename = create_info_file(game, "info_" + filename, seed, 0, DEFAULT_CONFIG);
    game.callMain([filename, info_filename]);

    while (isNaN(statusCode)) {
        await sleep(500);
    }
    // execution is finished

    var success = statusCode == 0;
    return success;
}

async function record(seed, filename, starting_bytes = []) {
    var statusCode = NaN;
    var game = await SM64({
        "canvas": document.querySelector("#canvas"), 
        "onExit": (e) => {
            statusCode = e;
        }
    });

    if (starting_bytes) {
        var stream = game.FS.open(filename, 'w+');
        game.FS.write(stream, starting_bytes, 0, starting_bytes.length, 0);
        game.FS.close(stream);
    }

    info_filename = create_info_file(game, "info_" + filename, seed, 1, DEFAULT_CONFIG);
    game.callMain([filename, info_filename]);

    while (isNaN(statusCode)) {
        await sleep(500);
    }
    // execution is finished

    var success = statusCode == 0;
    var solution_bytes = game.FS.open(filename).node.contents;
    console.log(success);
    console.log(solution_bytes);
    return [success, solution_bytes];
}

async function record_loop(seed, filename) {
    var success = false;
    //var starting_bytes = solution_22_array;
    var starting_bytes = [];
    while (!success) {
        [success, starting_bytes] = await record(seed, filename, starting_bytes);
    }
    return (starting_bytes);
}


record_loop(22, "awesome.m64");
