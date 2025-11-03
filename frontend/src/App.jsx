import { BrowserRouter, Routes, Route } from "react-router";
import Game from "./pages/Game";
import "./App.css";

function App() {
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