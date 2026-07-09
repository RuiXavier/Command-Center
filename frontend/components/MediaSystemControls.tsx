"use client";

import { vibrate, SystemState } from "../lib/utils";
import {
	SkipBack,
	Play,
	SkipForward,
	Moon,
	Sun,
	Lock,
	Bluetooth,
	BluetoothOff,
} from "lucide-react";

export function MediaSystemControls({
	sysState,
	sendCommand,
}: {
	sysState: SystemState;
	sendCommand: (ep: string, payload: Record<string, unknown>) => void;
}) {
	return (
		<div className="grid grid-cols-2 gap-4">
			{/* Media Player */}
			<section className="bg-white/5 backdrop-blur-2xl p-5 rounded-3xl border border-white/10 shadow-2xl flex flex-col justify-center">
				<h2 className="text-slate-400 mb-4 text-xs font-bold uppercase tracking-widest text-center">
					Media
				</h2>
				<div className="flex justify-between items-center gap-2">
					<button
						onClick={() => {
							vibrate(30);
							sendCommand("media", { action: "previous" });
						}}
						className="flex-1 bg-white/5 text-white hover:bg-white/10 h-14 rounded-2xl flex items-center justify-center active:scale-90 transition-all">
						<SkipBack size={20} fill="currentColor" />
					</button>
					<button
						onClick={() => {
							vibrate(50);
							sendCommand("media", { action: "play-pause" });
						}}
						className="flex-[1.2] bg-white text-black h-16 rounded-2xl flex items-center justify-center active:scale-90 transition-all shadow-[0_0_20px_rgba(255,255,255,0.2)]">
						<Play size={24} fill="currentColor" />
					</button>
					<button
						onClick={() => {
							vibrate(30);
							sendCommand("media", { action: "next" });
						}}
						className="flex-1 bg-white/5 text-white hover:bg-white/10 h-14 rounded-2xl flex items-center justify-center active:scale-90 transition-all">
						<SkipForward size={20} fill="currentColor" />
					</button>
				</div>
			</section>

			{/* System Toggles */}
			<section className="bg-white/5 backdrop-blur-2xl p-5 rounded-3xl border border-white/10 shadow-2xl flex flex-col justify-between">
				<h2 className="text-slate-400 mb-3 text-xs font-bold uppercase tracking-widest text-center">
					System
				</h2>

				<div className="grid grid-cols-2 gap-2 mb-2">
					<button
						onClick={() => {
							vibrate();
							sendCommand("theme", { name: "dark" });
						}}
						className="bg-slate-800 text-white h-10 rounded-xl flex items-center justify-center active:scale-90 transition-all shadow-inner">
						<Moon size={16} />
					</button>
					<button
						onClick={() => {
							vibrate();
							sendCommand("theme", { name: "light" });
						}}
						className="bg-slate-200 text-slate-900 h-10 rounded-xl flex items-center justify-center active:scale-90 transition-all shadow-inner">
						<Sun size={16} />
					</button>
				</div>

				{/* NEW: Prominent Bluetooth Button */}
				<button
					onClick={() => {
						vibrate(40);
						sendCommand("bluetooth", { action: "toggle" });
					}}
					className={`mb-2 w-full h-10 rounded-xl text-xs font-bold flex items-center justify-center gap-2 transition-all active:scale-95 border ${
						sysState.bluetooth_on
							? "bg-blue-500/10 text-blue-400 border-blue-500/20 hover:bg-blue-500/20"
							: "bg-black/20 text-slate-400 border-white/5 hover:bg-black/40"
					}`}>
					{sysState.bluetooth_on ? (
						<Bluetooth size={14} />
					) : (
						<BluetoothOff size={14} />
					)}
					<span className="truncate max-w-[100px]">
						{sysState.bluetooth_on ? sysState.bt_device : "Bluetooth Off"}
					</span>
				</button>

				<button
					onClick={() => {
						vibrate([50, 50, 50]);
						sendCommand("system", { action: "lock" });
					}}
					className="w-full bg-red-500/10 text-red-400 border border-red-500/20 hover:bg-red-500/20 h-10 rounded-xl text-xs font-bold uppercase tracking-wider active:scale-95 transition-all flex items-center justify-center gap-2">
					<Lock size={14} /> Lock
				</button>
			</section>
		</div>
	);
}
