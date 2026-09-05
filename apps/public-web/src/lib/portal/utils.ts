export function formatIdr(value: number): string {
	if (value === 0) return 'Rp0';
	return new Intl.NumberFormat('id-ID', {
		style: 'currency',
		currency: 'IDR',
		maximumFractionDigits: 0
	}).format(value);
}

export function formatDate(value: string): string {
	return new Intl.DateTimeFormat('id-ID', { dateStyle: 'medium' }).format(new Date(value));
}

export function formatDateTime(value: string): string {
	return new Intl.DateTimeFormat('id-ID', { dateStyle: 'medium', timeStyle: 'short' }).format(
		new Date(value)
	);
}

export function relativeDate(value: string): string {
	const days = Math.floor((Date.now() - new Date(value).getTime()) / 86_400_000);
	if (days <= 0) return 'today';
	if (days === 1) return '1 day ago';
	return `${days} days ago`;
}

export function truncateKey(key: string, visibleChars = 8): string {
	if (key.length <= visibleChars + 4) return key;
	return key.slice(0, visibleChars) + '••••' + key.slice(-4);
}

export function validateEmail(email: string): boolean {
	const trimmed = email.trim().toLowerCase();
	return trimmed.length > 0 && /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(trimmed);
}

export function sanitizeInput(value: string, maxLength = 255): string {
	return value.trim().slice(0, maxLength);
}
