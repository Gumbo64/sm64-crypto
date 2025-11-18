import { useEffect } from "react";
import { useRef, useState } from "react";
import { sm64_test, sm64_record, getROM, make_config } from "sm64-binds-frontend";
import { GamePadWeb, BlockChainClientWeb } from "sm64-crypto-browser";
import './Game.css';
import { GamePad } from "sm64-binds-frontend/src/scripts/sm64/Sm64Game";

// Array<GamePad> -> Array<GamePadWeb>
function map_solution_to_wasm(solution)  {
  return solution.map(e => GamePadWeb.new(e.button, e.stick_x, e.stick_y));
}


function Game() {
  const canvasRef = useRef(null);
  async function startGame() {

    let rng_config = make_config(64, 10*(60*30), 100, 5, 0.5, 0.5, 0.2);


    let rom_bytes = new Uint8Array(await getROM());
    let name = "NUMNUM64";
    let ticket = "sm64crypto-qvqdwfozjjojmxwnopewqfxp5jdsncii3sa7udovcgwbfp7sq5cqc3h4g73yws5224wgoeisynlkrrci4l4huznonzjopkgi2cpubgyv"

    let bc = await BlockChainClientWeb.new(rom_bytes, name, ticket);
    console.log("---------------------INITIALISED\n\n");
    let seed = await bc.start_mine();
    let solution = await sm64_record(canvasRef.current, seed, rng_config);
    // let solution = [new GamePad(0,0,0), new GamePad(0,80,80)];
    solution = map_solution_to_wasm(solution);

    await bc.submit_mine(seed, solution);

    // sm64_test(canvasRef.current);


  }

  useEffect(() => {
    startGame();
  }, []);

  return (
    <>
      <div className="container text-light" id="controls">
        <figure>
          <blockquote className="blockquote">
            <h1 className="display-6"><strong>Keyboard Controls</strong></h1>
          </blockquote>
          <figcaption className="blockquote-footer">
            You can use a controller!
          </figcaption>
          {/*<figcaption className="blockquote-footer"> You can save! Save is stored in local storage. </figcaption>*/}
          <figcaption className="blockquote-footer">
            Press page down to hide these instructions.
          </figcaption>
          <figcaption className="blockquote-footer">
            Press page up to see them again.
          </figcaption>
        </figure>
        <table className="table table-sm text-light" id="keyboard">
          <thead>
            <tr>
              <th scope="col">N64-Controller</th>
              <th scope="col">Keyboard</th>
              <th scope="col">Xbox Controller (xinput)</th>
              <th scope="col">Special Effect</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td>A</td>
              <td>I</td>
              <td>A</td>
            </tr>
            <tr>
              <td>B</td>
              <td>J</td>
              <td>X</td>
            </tr>
            <tr>
              <td>Z</td>
              <td>O</td>
              <td>LB</td>
            </tr>
            <tr>
              <td>R</td>
              <td>Right Shift</td>
              <td>RB</td>
            </tr>
            <tr>
              <td>C-Stick</td>
              <td>Arrow Keys</td>
              <td>Right Stick</td>
            </tr>
            <tr>
              <td>Start</td>
              <td>Space</td>
              <td>Start</td>
              <td>Resume Play (during playback)</td>
            </tr>
            <tr>
              <td>Dpad Up</td>
              <td>R</td>
              <td>Dpad Up</td>
              <td>Restart Game (and play back your inputs)</td>
            </tr>
            <tr>
              <td>Dpad Down (hold)</td>
              <td>Left Shift</td>
              <td>Dpad Down</td>
              <td>10x Speed</td>
            </tr>
          </tbody>
        </table>
      </div>
      <div id="container">
        <canvas ref={canvasRef} className="emscripten" id="canvas"></canvas>
      </div>
    </>
  );
}

export default Game;
