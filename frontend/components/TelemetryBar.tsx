"use client";

import { useState, useEffect } from "react";
import { SystemState } from "../lib/utils";
import {
	Wifi,
	WifiOff,
	Battery,
	BatteryMedium,
	BatteryLow,
	BatteryFull,
} from "lucide-react";

export function TelemetryBar({ sysState }: { sysState: SystemState }) {
	const [time, setTime] = useState("");

	useEffect(() => {
		const updateTime = () => {
			const now = new Date();
			setTime(
				now.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" }),
			);
		};
		updateTime();
		const interval = setInterval(updateTime, 1000);
		return () => clearInterval(interval);
	}, []);

	const renderBattery = () => {
		const bat = sysState.battery;
		if (bat > 90) return <BatteryFull size={14} className="text-emerald-400" />;
		if (bat > 50)
			return <BatteryMedium size={14} className="text-emerald-400" />;
		if (bat > 20) return <Battery size={14} className="text-yellow-400" />;
		return <BatteryLow size={14} className="text-red-500 animate-pulse" />;
	};

	return (
		<section className="relative bg-white/5 backdrop-blur-3xl px-5 py-3 rounded-2xl border border-white/10 shadow-lg mb-6 flex justify-between items-center text-slate-300">
			{/* Left: Network */}
			<div className="flex items-center gap-2 flex-1">
				{sysState.wifi_ssid !== "Disconnected" ? (
					<Wifi size={14} className="text-blue-400" />
				) : (
					<WifiOff size={14} className="text-slate-500" />
				)}
				<span className="text-xs font-medium tracking-wide max-w-[100px] truncate">
					{sysState.wifi_ssid}
				</span>
			</div>

			{/* Center: Time */}
			<div className="absolute left-1/2 -translate-x-1/2">
				<span className="text-xs font-bold tracking-widest text-slate-200">
					{time}
				</span>
			</div>

			{/* Right: Battery */}
			<div className="flex items-center justify-end gap-1.5 flex-1">
				<span className="text-xs font-medium font-mono tabular-nums pt-[1px]">
					{sysState.battery}%
				</span>
				{renderBattery()}
			</div>
		</section>
	);
}
