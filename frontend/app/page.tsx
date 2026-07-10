// frontend/app/page.tsx
"use client";

import { useState, useEffect, useCallback, useRef, useMemo } from "react";
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
import { TelemetryBar } from "../components/TelemetryBar";
import { NotificationFeed } from "../components/NotificationFeed";
import { AppLauncher } from "../components/AppLauncher";

// Replace your existing ModuleConfig and AppConfig types with this:

interface AppShortcut {
	name: string;
	command: string;
	icon: string;
}

type ModuleConfig =
	| { type: "telemetry" }
	| { type: "notifications" }
	| { type: "workspaces" }
	| { type: "media_system" }
	| {
			type: "slider";
			endpoint: "audio" | "brightness";
			label: string;
			color: string;
	  }
	| { type: "app_launcher"; apps: AppShortcut[] }; // NEW: Added App Launcher!

interface AppConfig {
	general: { theme: string };
	layout: ModuleConfig[];
}

export default function CommandCenter() {
	const [token, setToken] = useState<string | null>(null);
	const [sysState, setSysState] = useState<SystemState | null>(null);
	const [appConfig, setAppConfig] = useState<AppConfig | null>(null); // NEW: Config State

	const wsCooldown = useRef(0);
	const btCooldown = useRef(0);

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

	// NEW: Fetch Layout Config once on connection
	useEffect(() => {
		if (!token) return;
		fetch(`http://${window.location.hostname}:4000/api/layout`, {
			headers: { Authorization: `Bearer ${token}` },
		})
			.then((res) => res.json())
			.then((data) => setAppConfig(data))
			.catch(() => console.error("Failed to load config.toml"));
	}, [token]);

	const fetchState = useCallback(async () => {
		if (!token) return;
		try {
			const res = await fetch(
				`http://${window.location.hostname}:4000/api/state`,
				{
					headers: { Authorization: `Bearer ${token}` },
					cache: "no-store",
				},
			);
			if (res.ok) {
				const data = await res.json();
				setSysState((prev) => {
					if (!prev) return data;
					const now = Date.now();
					return {
						...data,
						active_workspace:
							now < wsCooldown.current
								? prev.active_workspace
								: data.active_workspace,
						bluetooth_on:
							now < btCooldown.current ? prev.bluetooth_on : data.bluetooth_on,
						bt_device:
							now < btCooldown.current ? prev.bt_device : data.bt_device,
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
		async (
			endpoint: string,
			payload: Record<string, unknown>,
		): Promise<void> => {
			if (endpoint === "workspace") {
				wsCooldown.current = Date.now() + 1000;
				setSysState((prev) =>
					prev ? { ...prev, active_workspace: payload.id as number } : null,
				);
			}
			if (endpoint === "bluetooth" && payload.action === "toggle") {
				btCooldown.current = Date.now() + 2500;
				setSysState((prev) =>
					prev
						? {
								...prev,
								bluetooth_on: !prev.bluetooth_on,
								bt_device: !prev.bluetooth_on ? "Searching..." : "Off",
							}
						: null,
				);
			}
			try {
				const res = await fetch(
					`http://${window.location.hostname}:4000/api/${endpoint}`,
					{
						method: "POST",
						headers: {
							"Content-Type": "application/json",
							Authorization: `Bearer ${token}`,
						},
						body: JSON.stringify(payload),
					},
				);
				if (!res.ok && res.status === 401) handleLogout();
			} catch (err: unknown) {
				console.error("Network error.");
			}
		},
		[token, handleLogout],
	);

	const volumeIcon = useMemo(
		() => (
			<button
				onClick={() => {
					vibrate();
					sendCommand("audio", { action: "mute" });
				}}
				className={`p-2 rounded-full transition-colors ${sysState?.is_muted ? "bg-red-500/20 text-red-400" : "text-slate-400"}`}>
				{sysState?.is_muted ? <VolumeX size={20} /> : <Volume2 size={20} />}
			</button>
		),
		[sysState?.is_muted, sendCommand],
	);

	const brightnessIcon = useMemo(
		() => (
			<div className="p-2 text-slate-400">
				<Sun size={20} />
			</div>
		),
		[],
	);

	if (!token || !sysState || !appConfig) {
		return (
			<main className="min-h-screen bg-black flex items-center justify-center p-6">
				<p className="text-slate-500 animate-pulse font-medium">
					Booting Command Center...
				</p>
			</main>
		);
	}

	// Separate the Telemetry bar from the rest of the grid so it spans the entire top natively
	const telemetryModule = appConfig.layout.find((m) => m.type === "telemetry");
	const masonryModules = appConfig.layout.filter((m) => m.type !== "telemetry");

	return (
		<main className="min-h-screen bg-[#0a0a0c] bg-[radial-gradient(ellipse_at_top,_var(--tw-gradient-stops))] from-slate-900 via-[#0a0a0c] to-black p-6 font-sans pb-24 text-slate-200 selection:bg-blue-500/30 overflow-x-hidden">
			<div className="max-w-6xl mx-auto">
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

				{/* 1. RENDER TELEMETRY AT THE TOP (Full Width) */}
				{telemetryModule && (
					<div className="mb-6">
						<TelemetryBar sysState={sysState} />
					</div>
				)}

				{/* 2. THE CSS COLUMNS MASONRY LAYOUT (Zero Gaps!) */}
				<div className="columns-1 lg:columns-3 gap-6 space-y-6">
					{masonryModules.map((module, index) => {
						// The 'break-inside-avoid' class ensures widgets aren't chopped in half across columns!
						return (
							<div
								key={index}
								className="break-inside-avoid inline-block w-full">
								{(() => {
									switch (module.type) {
										case "app_launcher":
											return (
												<AppLauncher
													apps={module.apps}
													sendCommand={sendCommand}
												/>
											);
										case "notifications":
											return (
												<NotificationFeed
													sysState={sysState}
													sendCommand={sendCommand}
												/>
											);
										case "workspaces":
											return (
												<WorkspaceGrid
													sysState={sysState}
													sendCommand={sendCommand}
												/>
											);
										case "media_system":
											return (
												<MediaSystemControls
													sysState={sysState}
													sendCommand={sendCommand}
												/>
											);
										case "slider":
											return (
												<section className="bg-white/5 backdrop-blur-2xl p-6 rounded-3xl border border-white/10 shadow-2xl">
													<h2 className="text-slate-400 text-xs font-bold uppercase tracking-widest mb-4">
														{module.label}
													</h2>
													<SmoothSlider
														serverValue={
															module.endpoint === "audio"
																? sysState.volume
																: sysState.brightness
														}
														endpoint={module.endpoint}
														trackColor={module.color}
														sendCommand={sendCommand}
														icon={
															module.endpoint === "audio"
																? volumeIcon
																: brightnessIcon
														}
													/>
												</section>
											);
										default:
											return null;
									}
								})()}
							</div>
						);
					})}
				</div>
			</div>
		</main>
	);
}

