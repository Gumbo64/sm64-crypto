import { useEffect } from "react";
import { useRef, useState, useContext } from "react";

function Home() {  
  return (
    <>
        <h1>Welcome to the Mario 64 Blockchain!</h1>
        <ul>
            <li>To start, mine a block in the Mine tab or check out replays in the Explorer</li>
            <li>To finish mining, you must collect 1 star!</li>
            <li>note: it may take time to sync blocks if the site freezes or the explorer is empty.
                If you don't provide a ticket, there won't be any initial blocks</li>
            <li>also i might train an AI using this data if im lucky</li>
        </ul>
    </>
  );
}

export default Home;
