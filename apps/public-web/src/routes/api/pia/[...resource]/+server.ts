import { json, type RequestHandler } from '@sveltejs/kit';
import { createPiaClient } from '$lib/server/pia-sdk.server';

export const GET: RequestHandler = async ({ params, url }) => {
	try {
		const client = createPiaClient();
		const symbol = url.searchParams.get('symbol')?.trim().toUpperCase();
		let data: unknown;

		switch (params.resource) {
			case 'market/prices':
				data = await client.market.getPrices();
				break;
			case 'news':
				data = await client.news.getNews();
				break;
			case 'economic/calendar':
				data = await client.economic.getCalendar();
				break;
			case 'rates/yield-curve':
				data = await client.fixedIncome.getYieldCurve();
				break;
			case 'fear-greed':
				data = await client.macro.getFearGreed();
				break;
			case 'options/summary':
				if (!symbol) return json({ error: 'symbol is required' }, { status: 400 });
				data = await client.options.getSummary(symbol);
				break;
			case 'options/chain':
				if (!symbol) return json({ error: 'symbol is required' }, { status: 400 });
				data = await client.options.getChain(symbol);
				break;
			case 'options/gex':
				if (!symbol) return json({ error: 'symbol is required' }, { status: 400 });
				data = await client.options.getGex(symbol);
				break;
			default:
				return json({ error: 'This feed is not available through the PIA SDK proxy' }, { status: 404 });
		}

		return json(data);
	} catch (error) {
		const statusCode =
			error && typeof error === 'object' && 'statusCode' in error &&
			typeof error.statusCode === 'number' && error.statusCode >= 400 && error.statusCode < 600
				? error.statusCode
				: 502;
		const message = error instanceof Error ? error.message : 'The PIA API request failed';
		return json({ error: message }, { status: statusCode });
	}
};
