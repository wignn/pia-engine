import audLogo from './assets/logo/forex/aud.png';
import cadLogo from './assets/logo/forex/cad.png';
import euroLogo from './assets/logo/forex/euro.png';
import gbpLogo from './assets/logo/forex/gbp.png';
import idrLogo from './assets/logo/forex/idr.png';
import jpyLogo from './assets/logo/forex/jpy.png';
import aaplLogo from './assets/logo/stock/us/appl.png';
import nvdaLogo from './assets/logo/stock/us/nvda.jpg';
import brentLogo from './assets/logo/metal/brent.png';
import goldLogo from './assets/logo/metal/gold.png';
import silverLogo from './assets/logo/metal/silver.png';
import wtiLogo from './assets/logo/metal/wti.png';

import asxLogo from './assets/logo/index/ASX.png';
import niftyLogo from './assets/logo/index/Nifty .png';
import dxyLogo from './assets/logo/index/dxy.png';
import hsiLogo from './assets/logo/index/hsi.png';
import ihsgLogo from './assets/logo/index/ihsg.png';
import jciLogo from './assets/logo/index/jci.png';
import kospiLogo from './assets/logo/index/kospi.png';
import sansexLogo from './assets/logo/index/sansex.png';
import snp500Logo from './assets/logo/index/snp500.png';
import ssecLogo from './assets/logo/index/ssec.png';
import stiLogo from './assets/logo/index/sti.png';

// Crypto
import bnbLogo from './assets/logo/crypto/bnb.png';
import btcLogo from './assets/logo/crypto/btc.png';
import ethLogo from './assets/logo/crypto/eth.png';
import solLogo from './assets/logo/crypto/sol.png';
import xautLogo from './assets/logo/crypto/xaut.png';

// Stock ID
import adroLogo from './assets/logo/stock/id/adro.png';
import asiiLogo from './assets/logo/stock/id/asii.png';
import bbcaLogo from './assets/logo/stock/id/bbca.png';
import bbniLogo from './assets/logo/stock/id/bbni.png';
import bbriLogo from './assets/logo/stock/id/bbri.png';
import bmriLogo from './assets/logo/stock/id/bmri.png';
import icbpLogo from './assets/logo/stock/id/icbp.png';
import mdkaLogo from './assets/logo/stock/id/mdka.png';
import tlkmLogo from './assets/logo/stock/id/tlkm.png';
import unvrLogo from './assets/logo/stock/id/unvr.png';

const logoMap: Record<string, string> = {
	// Forex
	AUDUSD: audLogo,
	USDCAD: cadLogo,
	EURUSD: euroLogo,
	GBPUSD: gbpLogo,
	USDIDR: idrLogo,
	USDJPY: jpyLogo,

	// Metal
	BRENT: brentLogo,
	UKOIL: brentLogo,
	XAUUSD: goldLogo,
	GOLD: goldLogo,
	XAGUSD: silverLogo,
	SILVER: silverLogo,
	WTI: wtiLogo,
	USOIL: wtiLogo,

	// Index
	ASX: asxLogo,
	ASX200: asxLogo,
	XJO: asxLogo,
	NIFTY: niftyLogo,
	NIFTY50: niftyLogo,
	'NIFTY 50': niftyLogo,
	DXY: dxyLogo,
	HSI: hsiLogo,
	IHSG: ihsgLogo,
	JCI: jciLogo,
	KOSPI: kospiLogo,
	SENSEX: sansexLogo,
	SPX: snp500Logo,
	SSEC: ssecLogo,
	STI: stiLogo,

	// Crypto
	BTCUSDT: btcLogo,
	ETHUSDT: ethLogo,
	SOLUSDT: solLogo,
	BNBUSDT: bnbLogo,
	PAXGUSDT: xautLogo,

	// Stocks
	ADRO: adroLogo,
	ASII: asiiLogo,
	BBCA: bbcaLogo,
	BBNI: bbniLogo,
	BBRI: bbriLogo,
	BMRI: bmriLogo,
	ICBP: icbpLogo,
	MDKA: mdkaLogo,
	TLKM: tlkmLogo,
	UNVR: unvrLogo,
	AAPL: aaplLogo,
	APPL: aaplLogo,
	NVDA: nvdaLogo
};

export function getLocalLogo(symbol: string): string | null {
	const upper = symbol.toUpperCase().replace(/[^A-Z0-9]/g, '');

	// Try exact match first
	if (logoMap[upper]) return logoMap[upper];

	// Try partial matching or prefix match (e.g. AUDUSD, EURUSD, etc.)
	if (upper.includes('EUR')) return euroLogo;
	if (upper.includes('GBP')) return gbpLogo;
	if (upper.includes('JPY')) return jpyLogo;
	if (upper.includes('AUD')) return audLogo;
	if (upper.includes('CAD')) return cadLogo;
	if (upper.includes('IDR')) return idrLogo;

	if (upper.includes('XAU') || upper.includes('GOLD')) return goldLogo;
	if (upper.includes('XAG') || upper.includes('SILVER')) return silverLogo;
	if (upper.includes('BRENT') || upper.includes('UKOIL')) return brentLogo;
	if (upper.includes('WTI') || upper.includes('USOIL')) return wtiLogo;

	if (
		upper.includes('SPX') ||
		upper.includes('SPY') ||
		upper.includes('SNP500') ||
		upper.includes('S&P')
	)
		return snp500Logo;
	if (upper.includes('DXY') || upper.includes('USDOLLAR')) return dxyLogo;
	if (upper.includes('IHSG')) return ihsgLogo;
	if (upper.includes('JCI') || upper.includes('COMPOSITE')) return jciLogo;
	if (upper.includes('HSI') || upper.includes('HANGSENG')) return hsiLogo;
	if (upper.includes('ASX')) return asxLogo;
	if (upper.includes('NIFTY')) return niftyLogo;
	if (upper.includes('KOSPI')) return kospiLogo;
	if (upper.includes('SENSEX')) return sansexLogo;
	if (upper.includes('SSEC')) return ssecLogo;
	if (upper.includes('STI')) return stiLogo;

	// Crypto fallbacks
	if (upper.includes('BTC')) return btcLogo;
	if (upper.includes('ETH')) return ethLogo;
	if (upper.includes('SOL')) return solLogo;
	if (upper.includes('BNB')) return bnbLogo;
	if (upper.includes('PAXG')) return xautLogo;

	// Stock ID fallbacks
	if (upper.includes('ADRO')) return adroLogo;
	if (upper.includes('ASII')) return asiiLogo;
	if (upper.includes('BBCA')) return bbcaLogo;
	if (upper.includes('BBNI')) return bbniLogo;
	if (upper.includes('BBRI')) return bbriLogo;
	if (upper.includes('BMRI')) return bmriLogo;
	if (upper.includes('ICBP')) return icbpLogo;
	if (upper.includes('MDKA')) return mdkaLogo;
	if (upper.includes('TLKM')) return tlkmLogo;
	if (upper.includes('UNVR')) return unvrLogo;

	// Stock US fallbacks
	if (upper.includes('AAPL') || upper.includes('APPL')) return aaplLogo;
	if (upper.includes('NVDA')) return nvdaLogo;

	return null;
}
