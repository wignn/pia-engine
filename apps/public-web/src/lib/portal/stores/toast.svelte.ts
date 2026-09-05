interface Toast {
	id: number;
	message: string;
	type: 'success' | 'error' | 'info';
}

let toasts = $state<Toast[]>([]);
let nextId = 0;

export const toastStore = {
	get toasts() {
		return toasts;
	},

	add(message: string, type: Toast['type'] = 'info', durationMs = 4000) {
		const id = nextId++;
		toasts = [...toasts, { id, message, type }];
		setTimeout(() => {
			toasts = toasts.filter((t) => t.id !== id);
		}, durationMs);
	},

	success(message: string) {
		this.add(message, 'success');
	},

	error(message: string) {
		this.add(message, 'error', 6000);
	},

	info(message: string) {
		this.add(message, 'info');
	},

	dismiss(id: number) {
		toasts = toasts.filter((t) => t.id !== id);
	}
};
