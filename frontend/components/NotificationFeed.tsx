// frontend/components/NotificationFeed.tsx
"use client";

import { SystemState, vibrate } from "../lib/utils";
import { Bell, BellOff, Trash2 } from "lucide-react";

export function NotificationFeed({
	sysState,
	sendCommand,
}: {
	sysState: SystemState;
	sendCommand: (ep: string, payload: Record<string, unknown>) => void;
}) {
	const notifs = sysState.notifications;

	const clearAll = () => {
		vibrate(50);
		sendCommand("notifications", { action: "clear" });
	};

	const clearOne = (id: string) => {
		vibrate(20);
		sendCommand("notifications", { action: "clear_one", id });
	};

	if (!notifs) return null;

	return (
		<section className="bg-white/5 backdrop-blur-3xl p-5 rounded-3xl border border-white/10 shadow-lg mb-6">
			<div className="flex justify-between items-center mb-4">
				<h2 className="text-slate-400 text-xs font-bold uppercase tracking-widest flex items-center gap-2">
					<Bell size={14} /> Recent Alerts
				</h2>
				{notifs.length > 0 && (
					<button
						onClick={clearAll}
						className="text-slate-500 hover:text-white text-xs font-bold transition-colors">
						Clear All
					</button>
				)}
			</div>

			<div className="space-y-3 max-h-[250px] overflow-y-auto scrollbar-hide pr-1">
				{notifs.length === 0 ? (
					<div className="text-center py-6 text-slate-500 flex flex-col items-center gap-2">
						<BellOff size={24} className="opacity-50" />
						<p className="text-xs font-medium">No new notifications</p>
					</div>
				) : (
					notifs.map((n) => (
						<div
							key={n.id}
							className="group relative bg-black/20 hover:bg-black/40 p-3 rounded-xl border border-white/5 transition-colors">
							<div className="flex justify-between items-start mb-1">
								<span className="text-blue-400 text-[10px] font-bold uppercase tracking-wider">
									{n.app_name}
								</span>
								<button
									onClick={() => clearOne(n.id)}
									className="text-slate-600 hover:text-red-400 opacity-0 group-hover:opacity-100 transition-opacity">
									<Trash2 size={14} />
								</button>
							</div>
							<h3 className="text-sm font-bold text-slate-200 leading-tight mb-1">
								{n.summary}
							</h3>
							{n.body && (
								<p className="text-xs text-slate-400 line-clamp-2 leading-relaxed">
									{n.body}
								</p>
							)}
						</div>
					))
				)}
			</div>
		</section>
	);
}
