function sleep(time) {
  return new Promise((resolve) => setTimeout(resolve, time));
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
        var stream = FS.open(filename, 'w+');
        game.FS.write(stream, solution_bytes, 0, solution_bytes.length, 0);
        game.FS.close(stream);
    }


    game.callMain([seed, filename, "0"]);
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

    game.callMain([seed, filename, "1"]);
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

// runGame("22", "epic.m64", "1", );



async function record_loop() {
    var success = false;
    var starting_bytes = solution_22_array;
    while (!success) {
        [success, starting_bytes] = await record("22", "awesome.m64", starting_bytes);
    }

}
record_loop();
