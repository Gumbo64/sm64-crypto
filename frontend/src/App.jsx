import { BrowserRouter, Routes, Route } from "react-router";
import React, { useState, useEffect } from 'react';
import Game from "./pages/Game";
import "./App.css";

import FileUpload from "./components/FileUpload.jsx";

function App() {
  const [hasRom, setHasRom] = useState(false);
  if (!hasRom) {
    return (
      <FileUpload setHasRom={setHasRom} />   
    );
  }

  return (
    <>
      <main>
        <BrowserRouter>
            <Routes>
              <Route path="/sm64-crypto" element={<Game />}/>
            </Routes>
        </BrowserRouter>
      </main>
    </>
  );
}

export default App;