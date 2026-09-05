interface RevealOptions {
	delay?: number;
	duration?: number;
	threshold?: number;
	y?: number;
}

export function reveal(
	node: HTMLElement,
	options: RevealOptions = {}
) {
	const { delay = 0, duration = 700, threshold = 0.1, y = 30 } = options;
	const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;

	if (reducedMotion) {
		node.style.opacity = '1';
		return { destroy() {} };
	}

	node.style.opacity = '0';
	node.style.transform = `translateY(${y}px)`;
	node.style.transition = `opacity ${duration}ms cubic-bezier(0.16, 1, 0.3, 1) ${delay}ms, transform ${duration}ms cubic-bezier(0.16, 1, 0.3, 1) ${delay}ms`;

	const observer = new IntersectionObserver(
		(entries) => {
			if (entries[0]?.isIntersecting) {
				node.style.opacity = '1';
				node.style.transform = 'translateY(0)';
				observer.unobserve(node);
			}
		},
		{ threshold }
	);

	observer.observe(node);

	return {
		destroy() {
			observer.disconnect();
		}
	};
}
