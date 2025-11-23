import React, { useEffect, useRef, useContext, useState } from 'react';

import { BlockChainClientWeb, GamePadWeb } from "sm64-crypto-browser";
import { GameConfig, RngConfig, sm64_playback } from "sm64-binds-frontend";
import { BlockchainContext } from '../context/BlockchainContext';
import { sm64_evaluate } from "sm64-binds-frontend";

function BlockWindow({ block }) {  
    const canvasRef = useRef(null);
    const [playing, setPlaying] = useState(false);

    if (block == null) {
        return (
            <h1>Null block</h1>
        )
    }

    async function play_solution() {
        if (!playing) {
            // let max_solution_time = BlockChainClientWeb.get_max_solution_time();
            // let rng_and_seed = block.calc_rng_and_seed();
            // let seed = rng_and_seed.seed;
            // let game_config = new GameConfig(max_solution_time, rng_and_seed);

            let seed = NaN;
            let game_config = new GameConfig(NaN, null);
            setPlaying(true);
            await sm64_playback(canvasRef.current, block.solution, false, seed, game_config, 2, 2);
            setPlaying(false);
        }

    }


    return (
        <div>
            <h1>{block.miner_name}'s Block</h1>
            <ul>
                <li>Block height: {block.block_height}</li>
                <li>Previous hash: {block.prev_hash}</li>
                <li>Timestamp: {block.timestamp}</li>
                {/* <li>Miner name: {block.miner_name}</li> */}
                {/* <li>Solution length: {block.solution.length}</li> */}
                <li><button onClick={play_solution}>Play solution</button></li>
                <li>
                    <div id="container">
                        <canvas ref={canvasRef} className="sm64canvas_small" id={playing && "canvas"}></canvas>
                    </div>
                </li>
            </ul>
        </div>
    );
}

export default BlockWindow;
