import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.jsx'

createRoot(document.getElementById('root')).render(
  // Turn off StrictMode to prevent React from double-rendering the game
  // which causes audio and graphical glitches
  //<StrictMode>
    <App />
  //</StrictMode>,
)
