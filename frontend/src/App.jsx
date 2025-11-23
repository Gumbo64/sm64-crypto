// App.js
import { BrowserRouter, Routes, Route } from "react-router";
import React, { useState, useRef, useContext } from 'react';
import Mine from "./pages/Mine";
import "./App.css";
import { FileUpload } from "sm64-binds-frontend";
import AppNavbar from "./components/AppNavbar";
import 'bootstrap/dist/css/bootstrap.min.css';
import { BlockchainProvider, BlockchainContext } from './context/BlockchainContext';
import Explorer from "./pages/Explorer";
import Home from "./pages/Home";

function App() {
  return (
    <BlockchainProvider>
      <BrowserRouter>
        <AppNavbar />
        <MainContent />
      </BrowserRouter>
    </BlockchainProvider>
  );
}

const MainContent = () => {
  const { hasRom, setHasRom } = useContext(BlockchainContext);
  
  if (!hasRom) {
    return <FileUpload setHasRom={setHasRom} />;
  }

  return (
    <main>
      <Routes>
        <Route path="/sm64-crypto/" element={<Home />} />
        <Route path="/sm64-crypto/mine" element={<Mine />} />
        <Route path="/sm64-crypto/explorer" element={<Explorer />} />
      </Routes>
    </main>
  );
};

export default App;
