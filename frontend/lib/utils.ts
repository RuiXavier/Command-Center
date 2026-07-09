// frontend/lib/utils.ts

export interface Notification {
	id: string;
	app_name: string;
	summary: string;
	body: string;
}

export interface SystemState {
	volume: number;
	is_muted: boolean;
	brightness: number;
	workspaces: number[];
	active_workspace: number;
	battery: number;
	wifi_ssid: string;
	bluetooth_on: boolean;
	bt_device: string;
	// --- NEW ---
	notifications: Notification[];
}

export const vibrate = (pattern: number | number[] = 50) => {
	if (typeof window !== "undefined" && navigator.vibrate) {
		navigator.vibrate(pattern);
	}
};
