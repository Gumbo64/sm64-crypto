import { useEffect } from "react";
import { useRef, useState, useContext } from "react";

import './Game.css';

function Home() {  
  return (
    <>
        <h1>Welcome to the Mario 64 Blockchain!</h1>
        <h3>To start, mine a block in the Game tab or check out replays in the Explorer</h3>
        <h3>note: it may take time to sync blocks if the site freezes or the explorer is empty</h3>
    </>
  );
}

export default Home;
