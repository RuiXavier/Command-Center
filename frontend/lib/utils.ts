export interface SystemState {
	volume: number;
	is_muted: boolean;
	brightness: number;
	workspaces: number[];
	active_workspace: number;
	// --- NEW ---
	battery: number;
	wifi_ssid: string;
	bluetooth_on: boolean;
	bt_device: string;
}

export const vibrate = (pattern: number | number[] = 50) => {
	if (typeof window !== "undefined" && navigator.vibrate) {
		navigator.vibrate(pattern);
	}
};

