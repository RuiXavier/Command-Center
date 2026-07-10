"use client";

import { useState, useMemo, useEffect } from "react";
import {
	Terminal,
	Globe,
	Folder,
	Code,
	Play,
	Search,
	ChevronRight,
	AppWindow,
} from "lucide-react";
import { vibrate } from "../lib/utils";

interface AppShortcut {
	name: string;
	command: string;
	icon: string;
}

export function AppLauncher({
	apps: tomlApps,
	sendCommand,
}: {
	apps: AppShortcut[];
	sendCommand: (ep: string, payload: Record<string, unknown>) => void;
}) {
	const [query, setQuery] = useState("");
	const [systemApps, setSystemApps] = useState<AppShortcut[]>([]);

	// Fetch all Linux apps from the Rust daemon on mount
	useEffect(() => {
		fetch(`http://${window.location.hostname}:4000/api/apps`)
			.then((res) => res.json())
			.then((data) => setSystemApps(data))
			.catch((err) => console.error("Failed to load system apps", err));
	}, []);

	const getIcon = (iconName: string) => {
		// 1. Check if it's one of your manual TOML overrides
		switch (iconName.toLowerCase()) {
			case "terminal":
				return (
					<Terminal
						size={20}
						className="text-slate-500 group-hover:text-blue-400"
					/>
				);
			case "browser":
				return (
					<Globe
						size={20}
						className="text-slate-500 group-hover:text-blue-400"
					/>
				);
			case "files":
				return (
					<Folder
						size={20}
						className="text-slate-500 group-hover:text-blue-400"
					/>
				);
			case "code":
				return (
					<Code
						size={20}
						className="text-slate-500 group-hover:text-blue-400"
					/>
				);
		}

		// 2. If it has a real Linux icon, fetch it from our Rust daemon!
		if (iconName && iconName !== "system") {
			const iconUrl = `http://${window.location.hostname}:4000/api/icon/${encodeURIComponent(iconName)}`;
			return (
				<img
					src={iconUrl}
					alt={iconName}
					className="w-5 h-5 object-contain opacity-70 group-hover:opacity-100 transition-opacity"
					onError={(e) => {
						// Hide the broken image box if the specific icon isn't found in standard paths
						(e.target as HTMLImageElement).style.display = "none";
					}}
				/>
			);
		}

		// 3. Absolute fallback
		return (
			<AppWindow
				size={20}
				className="text-slate-500 group-hover:text-blue-400"
			/>
		);
	};

	const filteredApps = useMemo(() => {
		// 1. If there is no search query, ONLY return the pinned TOML apps
		if (!query) return tomlApps || [];

		// 2. If the user is searching, merge everything together to search through
		const allApps = [...(tomlApps || [])];
		const pinnedNames = new Set(allApps.map((a) => a.name.toLowerCase()));

		systemApps.forEach((sysApp) => {
			if (!pinnedNames.has(sysApp.name.toLowerCase())) {
				allApps.push(sysApp);
			}
		});

		const q = query.toLowerCase();
		return allApps.filter(
			(a) =>
				a.name.toLowerCase().includes(q) || a.command.toLowerCase().includes(q),
		);
	}, [tomlApps, systemApps, query]);

	const executeCommand = (cmd: string) => {
		vibrate(40);
		sendCommand("execute", { command: cmd });
		setQuery("");
	};

	const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
		if (e.key === "Enter") {
			e.preventDefault();
			if (filteredApps.length > 0) executeCommand(filteredApps[0].command);
			else if (query.trim()) executeCommand(query);
		}
	};

	return (
		<section className="bg-[#0f0f13]/80 backdrop-blur-3xl rounded-3xl border border-white/10 shadow-2xl flex flex-col h-full overflow-hidden max-h-[400px]">
			<div className="relative border-b border-white/5 bg-white/5 shrink-0">
				<div className="absolute inset-y-0 left-0 pl-5 flex items-center pointer-events-none">
					<Search size={18} className="text-blue-400" />
				</div>
				<input
					type="text"
					value={query}
					onChange={(e) => setQuery(e.target.value)}
					onKeyDown={handleKeyDown}
					placeholder="Search apps or run command..."
					className="w-full bg-transparent py-5 pl-12 pr-5 text-sm font-medium text-slate-200 placeholder:text-slate-500 focus:outline-none focus:bg-white/[0.02] transition-colors"
					autoComplete="off"
					spellCheck="false"
				/>
			</div>

			<div className="flex-1 overflow-y-auto p-2 space-y-1 scrollbar-hide">
				{filteredApps.length > 0 ? (
					filteredApps.map((app, i) => (
						<button
							key={i}
							onClick={() => executeCommand(app.command)}
							className="w-full text-left px-4 py-3 rounded-xl hover:bg-white/10 flex items-center justify-between group transition-colors focus:outline-none focus:bg-white/10">
							<div className="flex items-center gap-4">
								<div className="text-slate-500 group-hover:text-blue-400 transition-colors">
									{getIcon(app.icon)}
								</div>
								<div>
									<div className="text-sm font-bold text-slate-200 group-hover:text-white transition-colors">
										{app.name}
									</div>
									<div className="text-[10px] text-slate-500 font-mono tracking-wider">
										{app.command}
									</div>
								</div>
							</div>
							<ChevronRight
								size={14}
								className="text-slate-600 opacity-0 group-hover:opacity-100 transition-all -translate-x-2 group-hover:translate-x-0"
							/>
						</button>
					))
				) : (
					<div className="px-4 py-6 text-center">
						<div className="text-sm font-bold text-slate-300 mb-1">
							Raw Command Mode
						</div>
						<div className="text-xs text-slate-500 font-mono">
							Press Enter to execute:{" "}
							<span className="text-emerald-400">{query}</span>
						</div>
					</div>
				)}
			</div>
		</section>
	);
}

