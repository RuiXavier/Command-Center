"use client";

import { useState, useEffect } from "react";
import { SystemState, vibrate } from "../lib/utils";
import {
	Wifi,
	WifiOff,
	Bluetooth,
	BluetoothOff,
	Battery,
	BatteryMedium,
	BatteryLow,
	BatteryFull,
} from "lucide-react";

type SendCommand = (command: string, payload?: Record<string, unknown>) => void;

export function TelemetryBar({
	sysState,
	sendCommand,
}: {
	sysState: SystemState;
	sendCommand: SendCommand;
}) {
	const [time, setTime] = useState("");

	// Self-updating clock isolated from the polling loop
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

	// Determine battery icon based on charge
	const renderBattery = () => {
		const bat = sysState.battery;
		if (bat > 90) return <BatteryFull size={16} className="text-emerald-400" />;
		if (bat > 50)
			return <BatteryMedium size={16} className="text-emerald-400" />;
		if (bat > 20) return <Battery size={16} className="text-yellow-400" />;
		return <BatteryLow size={16} className="text-red-500 animate-pulse" />;
	};

	const toggleBluetooth = () => {
		vibrate(40);
		sendCommand("bluetooth", { action: "toggle" });
	};

	return (
		<section className="bg-white/5 backdrop-blur-3xl px-5 py-3 rounded-2xl border border-white/10 shadow-lg mb-6 flex justify-between items-center text-slate-300">
			{/* Network & BT */}
			<div className="flex items-center gap-4">
				<div className="flex items-center gap-2">
					{sysState.wifi_ssid !== "Disconnected" ? (
						<Wifi size={16} className="text-blue-400" />
					) : (
						<WifiOff size={16} className="text-slate-500" />
					)}
					<span className="text-xs font-bold max-w-[100px] truncate">
						{sysState.wifi_ssid}
					</span>
				</div>
				<div className="w-[1px] h-4 bg-white/10" /> {/* Divider */}
				<div className="flex items-center justify-between mb-2">
					<button
						onClick={toggleBluetooth}
						className="flex items-center gap-2 transition-transform active:scale-95">
						{sysState.bluetooth_on ? (
							<Bluetooth size={16} className="text-blue-400" />
						) : (
							<BluetoothOff size={16} className="text-slate-500" />
						)}
						<span className="text-xs font-bold text-slate-400">
							{sysState.bluetooth_on ? sysState.bt_device : "Bluetooth Off"}
						</span>
					</button>
				</div>
			</div>

			{/* Time & Battery */}
			<div className="flex items-center gap-4">
				<span className="text-sm font-bold tracking-wider text-white">
					{time}
				</span>
				<div className="w-[1px] h-4 bg-white/10" /> {/* Divider */}
				<div className="flex items-center gap-1.5">
					<span className="text-xs font-bold font-mono">
						{sysState.battery}%
					</span>
					{renderBattery()}
				</div>
			</div>
		</section>
	);
}

