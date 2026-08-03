/**
 * Application entry point.
 * Mounts the React root into the #root div defined in index.html.
 */

import React from 'react';
import ReactDOM from 'react-dom/client';
import { App } from './App';
import './App.css';

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
