import {record_loop} from "./index.js"

import './App.css'

function App() {

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
            <th scope="col">N64-Controller</th>
            <th scope="col">Keyboard</th>
            <th scope="col">Xbox Controller (xinput)</th>
            <th scope="col">Special Effect</th>

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
              <td>Q</td>
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
        <canvas className="emscripten" id="canvas"></canvas>
      </div>
    </>
  )
}

export default App
