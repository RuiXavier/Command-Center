// frontend/lib/utils.ts

export interface SystemState {
	volume: number;
	is_muted: boolean;
	brightness: number;
	workspaces: number[];
	active_workspace: number;
}

export const vibrate = (pattern: number | number[] = 50) => {
	if (typeof window !== "undefined" && navigator.vibrate) {
		navigator.vibrate(pattern);
	}
};
