// frontend/app/page.tsx
"use client";

import { useState, useEffect, useCallback, useRef } from "react";
import {
	Volume2,
	VolumeX,
	Sun,
	MonitorSmartphone,
	PowerOff,
} from "lucide-react";
import { SystemState, vibrate } from "../lib/utils";
import { SmoothSlider } from "../components/SmoothSlider";
import { WorkspaceGrid } from "../components/WorkspaceGrid";
import { MediaSystemControls } from "../components/MediaSystemControls";

export default function CommandCenter() {
	const [token, setToken] = useState<string | null>(null);
	const [sysState, setSysState] = useState<SystemState | null>(null);

	// Cooldown so workspace buttons don't rubberband during network latency
	const wsCooldown = useRef(0);

	useEffect(() => {
		const initializeAuth = async () => {
			try {
				const rawUrl = window.location.href;
				let extractedToken: string | null = null;
				if (rawUrl.includes("?token="))
					extractedToken = rawUrl.split("?token=")[1].split("&")[0];

				if (extractedToken) {
					localStorage.setItem("cc-token", extractedToken);
					window.history.replaceState(null, "", window.location.pathname);
					setToken(extractedToken);
				} else {
					const savedToken = localStorage.getItem("cc-token");
					if (savedToken) setToken(savedToken);
				}
			} catch (err: unknown) {
				console.error("Init Error");
			}
		};
		initializeAuth();
	}, []);

	const fetchState = useCallback(async () => {
		if (!token) return;
		try {
			const ip = window.location.hostname;
			const res = await fetch(`http://${ip}:4000/api/state`, {
				headers: { Authorization: `Bearer ${token}` },
			});
			if (res.ok) {
				const data = await res.json();
				setSysState((prev) => {
					if (!prev) return data;
					return {
						...data,
						active_workspace:
							Date.now() < wsCooldown.current
								? prev.active_workspace
								: data.active_workspace,
					};
				});
			}
		} catch (err: unknown) {}
	}, [token]);

	useEffect(() => {
		if (!token) return;
		fetchState();
		const interval = setInterval(fetchState, 500);
		return () => clearInterval(interval);
	}, [token, fetchState]);

	const handleLogout = useCallback(() => {
		vibrate([50, 100, 50]);
		localStorage.removeItem("cc-token");
		setToken(null);
	}, []);

	const sendCommand = useCallback(
		async (endpoint: string, payload: Record<string, unknown>) => {
			// If it's a workspace command, instantly update optimistic UI and block server override for 1s
			if (endpoint === "workspace") {
				wsCooldown.current = Date.now() + 1000;
				setSysState((prev) =>
					prev ? { ...prev, active_workspace: payload.id as number } : null,
				);
			}

			try {
				const ip = window.location.hostname;
				const res = await fetch(`http://${ip}:4000/api/${endpoint}`, {
					method: "POST",
					headers: {
						"Content-Type": "application/json",
						Authorization: `Bearer ${token}`,
					},
					body: JSON.stringify(payload),
				});
				if (!res.ok && res.status === 401) handleLogout();
			} catch (err: unknown) {
				console.error("Network error.");
			}
		},
		[token, handleLogout],
	);

	if (!token) {
		return (
			<main className="min-h-screen bg-black flex items-center justify-center p-6">
				<p className="text-slate-500 animate-pulse font-medium">
					Waiting for Connection...
				</p>
			</main>
		);
	}

	return (
		<main className="min-h-screen bg-[#0a0a0c] bg-[radial-gradient(ellipse_at_top,_var(--tw-gradient-stops))] from-slate-900 via-[#0a0a0c] to-black p-6 font-sans pb-24 text-slate-200 selection:bg-blue-500/30">
			<div className="max-w-md mx-auto space-y-6">
				{/* HEADER */}
				<header className="flex justify-between items-center mt-4 mb-8">
					<div className="flex items-center gap-3">
						<div className="bg-blue-500/10 p-2 rounded-xl text-blue-400">
							<MonitorSmartphone size={24} />
						</div>
						<div>
							<h1 className="text-xl font-bold text-white tracking-tight">
								Command Center
							</h1>
							<p className="text-xs font-medium text-emerald-400 flex items-center gap-1.5">
								<span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span>{" "}
								Connected
							</p>
						</div>
					</div>
					<button
						onClick={handleLogout}
						className="text-slate-500 hover:text-white p-2 rounded-xl bg-white/5 active:scale-95 transition-all">
						<PowerOff size={20} />
					</button>
				</header>

				<WorkspaceGrid sysState={sysState} sendCommand={sendCommand} />

				{/* ENVIRONMENT CONTROLS */}
				<section className="bg-white/5 backdrop-blur-2xl p-6 rounded-3xl border border-white/10 shadow-2xl space-y-8">
					<SmoothSlider
						serverValue={sysState?.volume || 0}
						endpoint="audio"
						trackColor="bg-white"
						sendCommand={sendCommand}
						icon={
							<button
								onClick={() => {
									vibrate();
									sendCommand("audio", { action: "mute" });
								}}
								className={`p-2 rounded-full transition-colors ${sysState?.is_muted ? "bg-red-500/20 text-red-400" : "text-slate-400"}`}>
								{sysState?.is_muted ? (
									<VolumeX size={20} />
								) : (
									<Volume2 size={20} />
								)}
							</button>
						}
					/>
					<SmoothSlider
						serverValue={sysState?.brightness || 0}
						endpoint="brightness"
						trackColor="bg-yellow-400"
						sendCommand={sendCommand}
						icon={
							<div className="p-2 text-slate-400">
								<Sun size={20} />
							</div>
						}
					/>
				</section>

				<MediaSystemControls sendCommand={sendCommand} />
			</div>
		</main>
	);
}
