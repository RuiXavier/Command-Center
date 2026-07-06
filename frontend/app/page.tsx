'use client';

import { useState, useEffect } from 'react';

export default function CommandCenter() {
  const [token, setToken] = useState<string | null>(null);
  const [authInput, setAuthInput] = useState('');
  const [status, setStatus] = useState('');

  // 1. Check for token on mount (Bulletproof URL parsing)
  useEffect(() => {
    // Using the full href is much more reliable across mobile browsers
    const currentUrl = new URL(window.location.href);
    const urlToken = currentUrl.searchParams.get('token');

    if (urlToken) {
      console.log("Token found in URL!");
      // Save it
      localStorage.setItem('cc-token', urlToken);
      setToken(urlToken);
      
      // Clean up the URL bar so the token isn't visible in your phone's history
      window.history.replaceState({}, document.title, currentUrl.pathname);
    } else {
      // Fallback: Check if we already logged in previously
      const savedToken = localStorage.getItem('cc-token');
      if (savedToken) setToken(savedToken);
    }
  }, []);

  // 2. Handle parsing the JSON from the QR code
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
      // Fallback: If you just pasted the raw token string
      localStorage.setItem('cc-token', authInput);
      setToken(authInput);
    }
  };

  const handleLogout = () => {
    localStorage.removeItem('cc-token');
    setToken(null);
  };

  // 3. The universal API caller
  const sendCommand = async (endpoint: string, payload: any) => {
    // IMPORTANT: In production, change this to your laptop's actual local IP!
    // Example: const LAPTOP_IP = '192.168.1.50';
    const LAPTOP_IP = '127.0.0.1'; 
    const url = `http://${LAPTOP_IP}:4000/api/${endpoint}`;

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
        if (res.status === 401) handleLogout(); // Boot user if token is rejected
      }
    } catch (err) {
      setStatus('Network error. Is the Rust daemon running?');
    }
    
    setTimeout(() => setStatus(''), 2000); // Clear status after 2s
  };

  // --- UI: Unauthenticated State ---
  if (!token) {
    return (
      <main className="min-h-screen bg-slate-900 flex flex-col items-center justify-center p-6">
        <div className="bg-slate-800 p-8 rounded-2xl shadow-xl w-full max-w-md">
          <h1 className="text-2xl font-bold text-white mb-6 text-center">Pair Device</h1>
          <p className="text-slate-400 mb-4 text-sm text-center">
            Paste the JSON payload from your QR code scanner below.
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
          {status && <p className="text-red-400 text-center mt-4">{status}</p>}
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

        {/* Theme Controls */}
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

        {/* Media Controls */}
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