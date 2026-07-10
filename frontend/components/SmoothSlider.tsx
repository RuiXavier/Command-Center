"use client";

import { useState, useRef, useEffect, memo } from "react";
import * as Slider from "@radix-ui/react-slider";

export const SmoothSlider = memo(function SmoothSlider({
	serverValue,
	endpoint,
	icon,
	trackColor,
	sendCommand,
}: {
	serverValue: number;
	endpoint: string;
	icon: React.ReactNode;
	trackColor: string;
	sendCommand: (ep: string, payload: Record<string, unknown>) => Promise<void>;
}) {
	const [value, setValue] = useState([serverValue]);
	const isInteracting = useRef(false);
	const timeoutRef = useRef<NodeJS.Timeout | null>(null);

	const pendingValue = useRef<number | null>(null);
	const isSending = useRef(false);

	useEffect(() => {
		if (!isInteracting.current) {
			setValue([serverValue]);
		}
	}, [serverValue]);

	useEffect(() => {
		return () => {
			if (timeoutRef.current) clearTimeout(timeoutRef.current);
		};
	}, []);

	const lockSync = () => {
		isInteracting.current = true;
		if (timeoutRef.current) clearTimeout(timeoutRef.current);
	};

	const unlockSync = () => {
		if (timeoutRef.current) clearTimeout(timeoutRef.current);
		timeoutRef.current = setTimeout(() => {
			isInteracting.current = false;
		}, 1500);
	};

	const triggerSend = async () => {
		if (isSending.current || pendingValue.current === null) return;

		isSending.current = true;

		while (pendingValue.current !== null) {
			const valToSend = pendingValue.current;
			pendingValue.current = null;
			await sendCommand(endpoint, { action: "set", value: valToSend });
		}

		isSending.current = false;
	};

	const handleValueChange = (newValues: number[]) => {
		lockSync();
		setValue(newValues);
		pendingValue.current = newValues[0];
		triggerSend();
	};

	const handleValueCommit = (newValues: number[]) => {
		setValue(newValues);
		pendingValue.current = newValues[0];
		triggerSend();
		unlockSync();
	};

	// --- THE FIX ---
	// Check if the TOML string is a Tailwind class (starts with bg-)
	// or a native Hex Code (like #eab308)
	const isTailwind = trackColor?.startsWith("bg-");

	return (
		<div className="w-full">
			<div className="flex justify-between items-center mb-4">
				{icon}
				<span className="text-sm font-bold text-slate-400 tabular-nums">
					{value[0]}%
				</span>
			</div>
			<Slider.Root
				className="relative flex w-full touch-none select-none items-center"
				value={value}
				max={100}
				step={1}
				onValueChange={handleValueChange}
				onValueCommit={handleValueCommit}>
				<Slider.Track className="relative h-12 w-full grow overflow-hidden rounded-2xl bg-white/10 shadow-inner">
					{/* NATIVE COLOR INJECTION */}
					<Slider.Range
						className={`absolute h-full ${isTailwind ? trackColor : ""}`}
						style={!isTailwind ? { backgroundColor: trackColor } : {}}
					/>
				</Slider.Track>
				<Slider.Thumb className="hidden" />
			</Slider.Root>
		</div>
	);
});
