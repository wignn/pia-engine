export interface ChartTheme {
	background: string;
	textColor: string;
	gridColor: string;
	borderColor: string;
	crosshairColor: string;
	up: string;
	down: string;
	neutral: string;
	areaFillTop: (color: string) => string;
	areaFillBottom: (color: string) => string;
}

const LIGHT: ChartTheme = {
	background: '#fffefa',
	textColor: '#625f59',
	gridColor: '#e7e4dc',
	borderColor: '#d8d4ca',
	crosshairColor: '#8e8a81',
	up: '#2f8f64',
	down: '#c85550',
	neutral: '#c86622',
	areaFillTop: (color) => withAlpha(color, 0.1),
	areaFillBottom: () => 'rgba(0, 0, 0, 0)'
};

const DARK: ChartTheme = {
	background: '#191916',
	textColor: '#b9b2a7',
	gridColor: '#292721',
	borderColor: '#3a3730',
	crosshairColor: '#847d72',
	up: '#3ba576',
	down: '#d9685f',
	neutral: '#e4863d',
	areaFillTop: (color) => withAlpha(color, 0.1),
	areaFillBottom: () => 'rgba(0, 0, 0, 0)'
};

function withAlpha(hex: string, alpha: number): string {
	const normalized = hex.replace('#', '');
	if (normalized.length !== 6) return hex;
	const r = parseInt(normalized.slice(0, 2), 16);
	const g = parseInt(normalized.slice(2, 4), 16);
	const b = parseInt(normalized.slice(4, 6), 16);
	return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}

export function getChartTheme(isDark: boolean): ChartTheme {
	return isDark ? DARK : LIGHT;
}

export function isDarkMode(): boolean {
	return typeof document !== 'undefined' && document.documentElement.classList.contains('dark');
}
