INSERT INTO market.symbol_exchange_map (symbol, exchange_code, provider_symbol, asset_type, name, currency)
VALUES
    ('IHSG', 'IDX', 'IDX:COMPOSITE', 'index', 'Jakarta Composite Index (IHSG)', 'IDR'),
    ('ANTM', 'IDX', 'IDX:ANTM', 'stock', 'Aneka Tambang', 'IDR'),
    ('PTBA', 'IDX', 'IDX:PTBA', 'stock', 'Bukit Asam', 'IDR'),
    ('INDF', 'IDX', 'IDX:INDF', 'stock', 'Indofood Sukses Makmur', 'IDR')
ON CONFLICT (symbol) DO NOTHING;

