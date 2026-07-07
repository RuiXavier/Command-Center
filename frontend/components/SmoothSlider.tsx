// frontend/components/SmoothSlider.tsx
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

	// --- THE LATCH ENGINE ---
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

	// This guarantees we NEVER send overlapping commands to Linux!
	const triggerSend = async () => {
		if (isSending.current || pendingValue.current === null) return;

		isSending.current = true;

		// Keep sending until the queue is completely empty
		while (pendingValue.current !== null) {
			const valToSend = pendingValue.current;
			pendingValue.current = null; // Clear the queue BEFORE sending

			// Await the network request so Linux can process it sequentially
			await sendCommand(endpoint, { action: "set", value: valToSend });
		}

		isSending.current = false;
	};

	const handleValueChange = (newValues: number[]) => {
		lockSync();
		setValue(newValues);

		// Queue the latest value and kickstart the engine
		pendingValue.current = newValues[0];
		triggerSend();
	};

	const handleValueCommit = (newValues: number[]) => {
		setValue(newValues);

		// Guarantee the absolute final value is queued and sent
		pendingValue.current = newValues[0];
		triggerSend();

		unlockSync();
	};

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
					<Slider.Range className={`absolute h-full ${trackColor}`} />
				</Slider.Track>
				<Slider.Thumb className="hidden" />
			</Slider.Root>
		</div>
	);
});
