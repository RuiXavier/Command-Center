// frontend/components/SmoothSlider.tsx
"use client";

import { useState, useRef, memo } from "react";
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
	sendCommand: (ep: string, payload: Record<string, unknown>) => void;
}) {
	// THE FIX: dragVal is temporary. When null, the slider is forced to show absolute server truth.
	const [dragVal, setDragVal] = useState<number | null>(null);

	const lastSend = useRef(0);
	const timeoutRef = useRef<NodeJS.Timeout | null>(null);

	const displayVal = dragVal !== null ? dragVal : serverValue;

	const handleChange = (values: number[]) => {
		const val = values[0];
		setDragVal(val);

		if (timeoutRef.current) clearTimeout(timeoutRef.current);

		const now = Date.now();
		if (now - lastSend.current > 300) {
			sendCommand(endpoint, { action: "set", value: val });
			lastSend.current = now;
		}
	};

	const handleCommit = (values: number[]) => {
		const val = values[0];
		setDragVal(val);
		sendCommand(endpoint, { action: "set", value: val });

		// 1.5 seconds after letting go, destroy the local state.
		// This forces the UI to perfectly sync with the laptop's actual volume.
		if (timeoutRef.current) clearTimeout(timeoutRef.current);
		timeoutRef.current = setTimeout(() => {
			setDragVal(null);
		}, 1500);
	};

	return (
		<div>
			<div className="flex justify-between items-center mb-4">
				{icon}
				<span className="text-sm font-bold text-slate-500">{displayVal}%</span>
			</div>
			<Slider.Root
				className="relative flex items-center select-none touch-none w-full h-12"
				value={[displayVal]}
				max={100}
				step={1}
				onValueChange={handleChange}
				onValueCommit={handleCommit}>
				<Slider.Track className="bg-black/50 relative grow rounded-2xl h-full overflow-hidden shadow-inner cursor-pointer">
					<Slider.Range className={`absolute h-full ${trackColor}`} />
				</Slider.Track>
			</Slider.Root>
		</div>
	);
});
