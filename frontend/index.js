function sleep(time) {
  return new Promise((resolve) => setTimeout(resolve, time));
}

async function runGame(seed, filename, record_mode, canvas) {
    let our_canvas = false;
    if (canvas == null) {
        our_canvas = true;
        canvas = document.createElement('canvas');
        canvas.width  = 640;
        canvas.height = 480;
        document.body.appendChild(canvas);
    }


    var statusCode = NaN;
    
    var game = await SM64({
        "canvas": canvas, 
        "onExit": (e) => {
            statusCode = e;
        }, 
        // "onAbort": () => {}
    });
    game.callMain([seed, filename, record_mode]);

    while (isNaN(statusCode)) {
        await sleep(500);
    }
    console.log("DONE!!!");
    console.log(statusCode)
    
    
    if (our_canvas) {
        document.body.removeChild(canvas);
    }
    
    return statusCode;
}
runGame("22", "epic.m64", "1", document.querySelector("#canvas"));