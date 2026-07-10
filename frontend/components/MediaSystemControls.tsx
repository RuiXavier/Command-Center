"use client";

import { useState, useEffect } from "react";
import {
	Play,
	Pause,
	SkipBack,
	SkipForward,
	Music,
	Clapperboard,
} from "lucide-react";
import { vibrate } from "../lib/utils";

interface MediaMetadata {
	player_name: string;
	title: string;
	artist: string;
	art_url: string;
	status: string;
}

export function MediaSystemControls({
	sendCommand,
}: {
	sysState?: any;
	sendCommand: (ep: string, payload: Record<string, unknown>) => void;
}) {
	const [mediaList, setMediaList] = useState<MediaMetadata[]>([]);
	const [activeIndex, setActiveIndex] = useState(0);

	useEffect(() => {
		const fetchMeta = async () => {
			try {
				const res = await fetch(
					`http://${window.location.hostname}:4000/api/media/current`,
				);
				if (res.ok) {
					const data: MediaMetadata[] = await res.json();
					setMediaList(data);
					// Auto-adjust index if a player is closed
					if (activeIndex >= data.length)
						setActiveIndex(Math.max(0, data.length - 1));
				}
			} catch (e) {}
		};

		fetchMeta();
		const interval = setInterval(fetchMeta, 1500);
		return () => clearInterval(interval);
	}, [activeIndex]);

	const currentMeta = mediaList[activeIndex] || null;

	if (!currentMeta) {
		return (
			<section className="bg-white/5 backdrop-blur-2xl p-5 rounded-3xl border border-white/10 shadow-2xl h-full flex flex-col items-center justify-center min-h-[280px]">
				<Music size={32} className="text-white/20 mb-3" />
				<div className="text-sm font-bold text-slate-500">No Media Playing</div>
			</section>
		);
	}

	const isPlaying = currentMeta.status.toLowerCase() === "playing";
	const artProxyUrl = currentMeta.art_url
		? `http://${window.location.hostname}:4000/api/media/art?url=${encodeURIComponent(currentMeta.art_url)}`
		: "";

	// DETECTION: Check if Stremio/MPV is the active player
	const isStremio =
		currentMeta.player_name.toLowerCase().includes("mpv") ||
		currentMeta.player_name.toLowerCase().includes("stremio");

	// Control function that targets the specific player
	const handleControl = (action: string) => {
		vibrate(30);
		sendCommand("media", { action, player: currentMeta.player_name });
	};

	return (
		<section
			className={`relative overflow-hidden p-5 rounded-3xl border shadow-2xl h-full flex flex-col group transition-all min-h-[280px] ${
				isStremio
					? "bg-purple-900/20 border-purple-500/30"
					: "bg-white/5 border-white/10"
			} backdrop-blur-2xl`}>
			{/* 1. Ambient Background */}
			{artProxyUrl && (
				<>
					<div
						className={`absolute inset-0 bg-cover bg-center opacity-30 group-hover:opacity-40 transition-opacity duration-700 blur-2xl ${isStremio ? "saturate-200" : "saturate-150"}`}
						style={{ backgroundImage: `url(${artProxyUrl})` }}
					/>
					<div className="absolute inset-0 bg-gradient-to-t from-[#0a0a0c]/95 via-[#0a0a0c]/50 to-transparent" />
				</>
			)}

			{/* 2. Header & Multi-Player Tabs */}
			<div className="relative z-10 flex justify-between items-center mb-4 shrink-0">
				<h2
					className={`text-xs font-bold uppercase tracking-widest flex items-center gap-2 drop-shadow-md ${isStremio ? "text-purple-300" : "text-slate-400"}`}>
					{isStremio ? (
						<Clapperboard size={14} className="text-purple-400" />
					) : (
						<Music size={14} className="text-emerald-400" />
					)}
					{isStremio ? "Stremio" : currentMeta.player_name.split(".")[0]}
				</h2>

				{/* Pagination Dots for Multiple Players */}
				{mediaList.length > 1 && (
					<div className="flex gap-1.5">
						{mediaList.map((_, i) => (
							<button
								key={i}
								onClick={() => {
									vibrate(10);
									setActiveIndex(i);
								}}
								className={`w-2 h-2 rounded-full transition-all ${i === activeIndex ? (isStremio ? "bg-purple-400 w-4" : "bg-emerald-400 w-4") : "bg-white/20"}`}
							/>
						))}
					</div>
				)}
			</div>

			{/* 3. Banner Image */}
			{artProxyUrl && (
				<div
					className={`relative z-10 w-full h-32 mb-5 rounded-xl overflow-hidden shadow-lg border shrink-0 bg-black/40 ${isStremio ? "border-purple-500/20 shadow-purple-900/20" : "border-white/10"}`}>
					<img
						src={artProxyUrl}
						alt="Media Art"
						className="w-full h-full object-cover"
						onError={(e) => {
							(e.target as HTMLImageElement).style.display = "none";
						}}
					/>
					<div className="absolute inset-0 bg-[radial-gradient(ellipse_at_center,_var(--tw-gradient-stops))] from-transparent via-black/10 to-black/60 mix-blend-overlay"></div>
				</div>
			)}

			{/* 4. Track Info */}
			<div className="relative z-10 flex flex-col items-center text-center gap-1 mb-5 mt-auto w-full px-2">
				<div
					className={`font-bold truncate w-full drop-shadow-md ${isStremio ? "text-lg text-white" : "text-base text-slate-100"}`}>
					{currentMeta.title}
				</div>

				{/* Hide Artist in Stremio Mode (since it's usually empty or a generic string) */}
				{!isStremio && (
					<div className="text-xs font-bold text-slate-400 truncate w-full uppercase tracking-wider drop-shadow-md">
						{currentMeta.artist || "Unknown Artist"}
					</div>
				)}
			</div>

			{/* 5. Playback Controls */}
			<div className="relative z-10 flex items-center justify-center gap-5 shrink-0 pb-1">
				<button
					onClick={() => handleControl("previous")}
					className="p-3 rounded-full bg-white/5 hover:bg-white/20 text-slate-200 transition-all active:scale-95 backdrop-blur-md">
					<SkipBack size={18} className="fill-current" />
				</button>

				<button
					onClick={() => handleControl("play-pause")}
					className={`p-4 rounded-full text-slate-900 transition-all shadow-xl active:scale-95 ${isStremio ? "bg-purple-400 hover:bg-purple-300 shadow-purple-900/50" : "bg-slate-100 hover:bg-white shadow-black/50"}`}>
					{isPlaying ? (
						<Pause size={20} className="fill-current" />
					) : (
						<Play size={20} className="fill-current ml-1" />
					)}
				</button>

				<button
					onClick={() => handleControl("next")}
					className="p-3 rounded-full bg-white/5 hover:bg-white/20 text-slate-200 transition-all active:scale-95 backdrop-blur-md">
					<SkipForward size={18} className="fill-current" />
				</button>
			</div>
		</section>
	);
}
