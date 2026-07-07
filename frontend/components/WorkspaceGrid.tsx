// frontend/components/WorkspaceGrid.tsx
"use client";

import { SystemState, vibrate } from "../lib/utils";

export function WorkspaceGrid({
	sysState,
	sendCommand,
}: {
	sysState: SystemState | null;
	sendCommand: (ep: string, payload: Record<string, unknown>) => void;
}) {
	const handleSwitch = (id: number) => {
		vibrate(40);
		sendCommand("workspace", { id });
	};

	return (
		<section className="bg-white/5 backdrop-blur-2xl p-5 rounded-3xl border border-white/10 shadow-2xl">
			<h2 className="text-slate-400 mb-4 text-xs font-bold uppercase tracking-widest">
				Workspaces
			</h2>
			<div className="flex gap-3 overflow-x-auto pb-2 scrollbar-hide">
				{sysState?.workspaces.map((ws) => (
					<button
						key={ws}
						onClick={() => handleSwitch(ws)}
						className={`min-w-[56px] h-14 rounded-2xl font-bold text-lg transition-all active:scale-90 flex items-center justify-center ${
							sysState.active_workspace === ws
								? "bg-blue-500 text-white shadow-[0_0_20px_rgba(59,130,246,0.4)]"
								: "bg-white/5 text-slate-400 hover:bg-white/10"
						}`}>
						{ws}
					</button>
				))}
				{(!sysState || sysState.workspaces.length === 0) && (
					<p className="text-slate-500 text-sm">Loading...</p>
				)}
			</div>
		</section>
	);
}
