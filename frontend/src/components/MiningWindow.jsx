import React, { useEffect, useRef, useContext } from 'react';

import { GamePadWeb } from "sm64-crypto-browser";
import { make_config } from "sm64-binds-frontend";
import { BlockchainContext } from '../context/BlockchainContext';
import { sm64_record } from "sm64-binds-frontend";

function init_config() {
    return make_config(64, 10*(60*30), 100, 5, 0.5, 0.5, 0.2);
}

// Array<GamePad> -> Array<GamePadWeb>
function map_solution_to_wasm(solution)  {
    return solution.map(e => GamePadWeb.new(e.button, e.stick_x, e.stick_y));
}

async function startMining(canvasRef, blockchain, rng_config, total_kill_signal = () => {false}) {
    console.log("---------------------INITIALISED\n\n");
    let isMining = true;

    async function kill_signal() {
        if (total_kill_signal()) {
            isMining = false;
            return true;
        }

        if (await blockchain.has_new_block()) {
            console.log("New block found, restarting game");
            return true;
        }
        return false;
    }

    while (isMining) {
        console.log("------------------ started mine\n\n");
        let seed = await blockchain.start_mine();
        let solution;
        try {
            solution = await sm64_record(canvasRef.current, seed, rng_config, kill_signal);
        } catch (error) {
            continue;
        }
        solution = map_solution_to_wasm(solution);
        await blockchain.submit_mine(seed, solution);
    }
}


function MiningWindow() {  
    const canvasRef = useRef(null);
    const { blockchain } = useContext(BlockchainContext);

    useEffect(() => {
        if (blockchain) {
            const initialUrl = window.location.href;

            let rngConfig = init_config();

            function endMining() {
                if (window.location.href !== initialUrl) {
                    console.log("Left the mining page, closing mining window")
                    return true;
                }
                return false;
            }

            startMining(canvasRef, blockchain, rngConfig, endMining);
        }
    }, [blockchain]);


    return (
        <div id="container">
            <canvas ref={canvasRef} className="emscripten" id="canvas"></canvas>
        </div>
    );
}

export default MiningWindow;
