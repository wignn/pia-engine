from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]
PANEL = ROOT / "apps/terminal/src/components/OrderBookPanel.tsx"
BACKEND = ROOT / "services/market-data/src/prices.rs"


class OrderBookProvenanceTests(unittest.TestCase):
    def test_terminal_does_not_generate_orderbook_or_trade_data(self):
        source = PANEL.read_text()
        self.assertNotIn("Math.random()", source)
        self.assertNotIn("Generate dynamic Level-2 Depth", source)
        self.assertNotIn("Generate mock real-time trade tape", source)
        self.assertNotIn("Math.sin", source)
        self.assertNotIn("Math.cos", source)
        self.assertNotIn("100.0", source)

    def test_backend_reports_depth_unavailable_instead_of_synthesizing(self):
        source = BACKEND.read_text()
        self.assertIn('"is_live": false', source)
        self.assertIn('"synthetic": false', source)
        self.assertIn('"depth_type": "unavailable"', source)
        self.assertIn('"bids": []', source)
        self.assertIn('"asks": []', source)
        self.assertNotIn('"depth_type": "quote_derived"', source)


if __name__ == "__main__":
    unittest.main()
