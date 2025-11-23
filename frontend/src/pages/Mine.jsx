import { useEffect } from "react";
import { useRef, useState, useContext } from "react";

import './Mine.css';
import MiningWindow from "../components/MiningWindow";

function Mine() {  

  return (
    <>
      <div className="container" id="controls">
        <figure>
          <h1><strong>Controls</strong></h1>
          <figcaption className="blockquote-footer">
            You can use a controller! (xinput/xbox etc)
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
              <td>Restart Mine (and play back your inputs)</td>
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
      <MiningWindow/>
    </>
  );
}

export default Mine;
