'use client';

import { useState, useEffect } from 'react';

export default function CommandCenter() {
  const [token, setToken] = useState<string | null>(null);
  const [authInput, setAuthInput] = useState('');
  const [status, setStatus] = useState('');
  const [backendIp, setBackendIp] = useState('127.0.0.1');
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    try {
      // 1. Grab IP
      setBackendIp(window.location.hostname);

      // 2. Extract Token
      const rawUrl = window.location.href;
      let extractedToken = null;
      
      if (rawUrl.includes('?token=')) {
        extractedToken = rawUrl.split('?token=')[1].split('&')[0];
      }

      if (extractedToken) {
        // Attempt to save and clean URL
        localStorage.setItem('cc-token', extractedToken);
        setToken(extractedToken);
        
        // Passing 'null' and an empty string is safer for strict mobile browsers
        window.history.replaceState(null, '', window.location.pathname);
      } else {
        // Fallback to local storage
        const savedToken = localStorage.getItem('cc-token');
        if (savedToken) setToken(savedToken);
      }
    } catch (err: any) {
      // If the mobile browser panics (e.g. strict local storage policies), catch it here!
      setStatus('Init Error: ' + (err.message || 'Unknown error occurred'));
    } finally {
      // THIS is the magic bullet. No matter what happens, exit the loading screen.
      setIsLoading(false);
    }
  }, []);

  const handleLogin = () => {
    try {
      const parsed = JSON.parse(authInput);
      if (parsed.token) {
        localStorage.setItem('cc-token', parsed.token);
        setToken(parsed.token);
      } else {
        setStatus('Invalid JSON format.');
      }
    } catch {
      localStorage.setItem('cc-token', authInput);
      setToken(authInput);
    }
  };

  const handleLogout = () => {
    try {
      localStorage.removeItem('cc-token');
    } catch (err) {
      // Ignore storage errors on logout
    }
    setToken(null);
  };

  const sendCommand = async (endpoint: string, payload: any) => {
    const url = `http://${backendIp}:4000/api/${endpoint}`;

    try {
      const res = await fetch(url, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`
        },
        body: JSON.stringify(payload)
      });

      if (res.ok) {
        setStatus('Command sent successfully!');
      } else {
        setStatus('Failed to authenticate or execute.');
        if (res.status === 401) handleLogout();
      }
    } catch (err) {
      setStatus('Network error. Is the Rust daemon running?');
    }
    
    setTimeout(() => setStatus(''), 2000);
  };

  // --- UI: Loading State ---
  if (isLoading) {
    return (
      <main className="min-h-screen bg-slate-900 flex items-center justify-center p-6 text-center">
        <p className="text-slate-500 animate-pulse text-lg">Establishing secure connection...</p>
      </main>
    );
  }

  // --- UI: Unauthenticated State ---
  if (!token) {
    return (
      <main className="min-h-screen bg-slate-900 flex flex-col items-center justify-center p-6">
        <div className="bg-slate-800 p-8 rounded-2xl shadow-xl w-full max-w-md">
          <h1 className="text-2xl font-bold text-white mb-6 text-center">Pair Device</h1>
          <p className="text-slate-400 mb-4 text-sm text-center">
            {status ? (
              <span className="text-red-400 font-mono text-xs">{status}</span>
            ) : (
              "Paste the JSON payload from your QR code scanner below."
            )}
          </p>
          <textarea 
            className="w-full bg-slate-900 text-green-400 p-4 rounded-xl border border-slate-700 focus:outline-none focus:border-blue-500 mb-4 font-mono text-sm h-32"
            value={authInput}
            onChange={(e) => setAuthInput(e.target.value)}
            placeholder='{"token": "..."}'
          />
          <button 
            onClick={handleLogin}
            className="w-full bg-blue-600 hover:bg-blue-500 text-white font-semibold py-3 rounded-xl transition-colors"
          >
            Authenticate
          </button>
        </div>
      </main>
    );
  }

  // --- UI: Authenticated State (The Dashboard) ---
  return (
    <main className="min-h-screen bg-slate-900 p-6 font-sans">
      <div className="max-w-md mx-auto">
        <div className="flex justify-between items-center mb-8">
          <h1 className="text-2xl font-bold text-white">Command Center</h1>
          <button onClick={handleLogout} className="text-slate-400 hover:text-white text-sm">Disconnect</button>
        </div>

        <section className="mb-8">
          <h2 className="text-slate-400 mb-4 text-sm font-semibold uppercase tracking-wider">Appearance</h2>
          <div className="grid grid-cols-2 gap-4">
            <button 
              onClick={() => sendCommand('theme', { name: 'dark' })}
              className="bg-slate-800 hover:bg-slate-700 border border-slate-700 text-white p-6 rounded-2xl transition-all shadow-lg active:scale-95"
            >
              🌙 Mocha Theme
            </button>
            <button 
              onClick={() => sendCommand('theme', { name: 'light' })}
              className="bg-slate-200 hover:bg-white text-slate-900 p-6 rounded-2xl transition-all shadow-lg active:scale-95 font-medium"
            >
              ☀️ Latte Theme
            </button>
          </div>
        </section>

        <section>
          <h2 className="text-slate-400 mb-4 text-sm font-semibold uppercase tracking-wider">Media</h2>
          <div className="flex justify-between bg-slate-800 p-2 rounded-2xl border border-slate-700">
            <button 
              onClick={() => sendCommand('media', { action: 'previous' })}
              className="flex-1 text-white hover:bg-slate-700 p-4 rounded-xl transition-colors active:bg-slate-600 text-2xl"
            >
              ⏮
            </button>
            <button 
              onClick={() => sendCommand('media', { action: 'play-pause' })}
              className="flex-1 text-white hover:bg-slate-700 p-4 rounded-xl transition-colors active:bg-slate-600 text-2xl"
            >
              ⏯
            </button>
            <button 
              onClick={() => sendCommand('media', { action: 'next' })}
              className="flex-1 text-white hover:bg-slate-700 p-4 rounded-xl transition-colors active:bg-slate-600 text-2xl"
            >
              ⏭
            </button>
          </div>
        </section>

        {status && (
          <div className="fixed bottom-6 left-6 right-6 bg-slate-800 border border-slate-700 text-white p-4 rounded-xl text-center shadow-2xl animate-pulse">
            {status}
          </div>
        )}
      </div>
    </main>
  );
}