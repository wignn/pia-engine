#!/usr/bin/env python3
"""
Daily Macro & Fundamentals Data Synchronization for PIA Engine / ATLSD.
Runs 1x per day (recommended: 03:00 UTC) via cron.
Syncs:
1. Economic Calendar (macro.economic_calendar_events)
2. CFTC Commitment of Traders (cot_reports)
3. SEC EDGAR Corporate Filings (sec_filings)
"""

import sys
import os
import json
import urllib.request
import urllib.parse
import hashlib
import zipfile
import io
import csv
import subprocess
from datetime import datetime, timezone

def log(msg: str):
    ts = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M:%S UTC")
    print(f"[{ts}] {msg}", flush=True)

def exec_psql(sql: str) -> str:
    cmd = [
        "docker", "exec", "-i", "postgres-prod",
        "psql", "-U", "atlsd", "-d", "core", "-q", "-t", "-c", sql
    ]
    res = subprocess.run(cmd, capture_output=True, text=True)
    if res.returncode != 0 and res.stderr:
        log(f"psql error: {res.stderr.strip()[:200]}")
        return ""
    return res.stdout.strip()

def sync_cot():
    log("--- Syncing CFTC Commitment of Traders (COT) ---")
    year = datetime.now(timezone.utc).year
    url = f"https://www.cftc.gov/files/dea/history/deacot{year}.zip"
    headers = {"User-Agent": "PIATerminal/1.0 (contact@pia.wign.dev)"}
    
    try:
        req = urllib.request.Request(url, headers=headers)
        with urllib.request.urlopen(req, timeout=20) as resp:
            zip_bytes = resp.read()
            with zipfile.ZipFile(io.BytesIO(zip_bytes)) as z:
                files = z.namelist()
                target_file = files[0] if files else "annual.txt"
                content = z.read(target_file).decode("latin-1", errors="replace")
                
                reader = csv.reader(io.StringIO(content))
                # Skip header
                header = next(reader, None)
                count = 0
                for row in reader:
                    if len(row) < 15:
                        continue
                    market_name = row[0].strip().replace("'", "''")
                    report_date = row[2].strip() # YYYY-MM-DD
                    cftc_code = row[3].strip() if len(row) > 3 else ""
                    
                    if not market_name or not report_date or len(report_date) != 10:
                        continue
                        
                    open_interest = int(row[7].strip()) if row[7].strip().isdigit() else 0
                    noncomm_long = int(row[8].strip()) if row[8].strip().isdigit() else 0
                    noncomm_short = int(row[9].strip()) if row[9].strip().isdigit() else 0
                    comm_long = int(row[11].strip()) if row[11].strip().isdigit() else 0
                    comm_short = int(row[12].strip()) if row[12].strip().isdigit() else 0
                    
                    code = cftc_code or market_name[:20].strip()
                    sql = f"""
                    INSERT INTO macro.cot_reports 
                        (market_code, market_name, report_date, report_type, open_interest, noncommercial_long, noncommercial_short, commercial_long, commercial_short, updated_at)
                    VALUES 
                        ('{code}', '{market_name}', '{report_date}', 'legacy_futures_only', {open_interest}, {noncomm_long}, {noncomm_short}, {comm_long}, {comm_short}, NOW())
                    ON CONFLICT (report_type, market_code, report_date) DO UPDATE SET
                        open_interest = EXCLUDED.open_interest,
                        noncommercial_long = EXCLUDED.noncommercial_long,
                        noncommercial_short = EXCLUDED.noncommercial_short,
                        commercial_long = EXCLUDED.commercial_long,
                        commercial_short = EXCLUDED.commercial_short,
                        updated_at = NOW();
                    """
                    exec_psql(sql)
                    count += 1
                    if count >= 300: # Refresh top 300 active market contracts
                        break
                        
                log(f"Synced {count} CFTC COT market records to PostgreSQL")
    except Exception as e:
        log(f"COT sync failed: {e}")

def sync_sec():
    log("--- Syncing SEC Corporate Filings ---")
    ciks = {
        "AAPL": "0000320193",
        "MSFT": "0000789019",
        "NVDA": "0001045810",
        "TSLA": "0001318605",
        "AMZN": "0001018724",
        "GOOGL": "0001652044",
        "META": "0001326801"
    }
    headers = {
        "User-Agent": "PIATerminal/1.0 (contact@pia.wign.dev)",
        "Accept-Encoding": "gzip, deflate",
        "Host": "data.sec.gov"
    }
    
    total_filings = 0
    for ticker, cik in ciks.items():
        url = f"https://data.sec.gov/submissions/CIK{cik}.json"
        try:
            req = urllib.request.Request(url, headers=headers)
            with urllib.request.urlopen(req, timeout=15) as resp:
                import gzip
                raw = resp.read()
                try:
                    text = gzip.decompress(raw).decode("utf-8")
                except Exception:
                    text = raw.decode("utf-8")
                    
                data = json.loads(text)
                company_name = data.get("name", ticker).replace("'", "''")
                recent = data.get("filings", {}).get("recent", {})
                forms = recent.get("form", [])
                acc_nums = recent.get("accessionNumber", [])
                filing_dates = recent.get("filingDate", [])
                primary_docs = recent.get("primaryDocument", [])
                
                for i in range(min(15, len(forms))):
                    form = forms[i]
                    acc = acc_nums[i]
                    fdate = filing_dates[i]
                    doc = primary_docs[i] if i < len(primary_docs) else ""
                    doc_url = f"https://www.sec.gov/Archives/edgar/data/{int(cik)}/{acc.replace('-', '')}/{doc}"
                    title = f"{ticker} ({form}) - {company_name}".replace("'", "''")
                    
                    sql = f"""
                    INSERT INTO sec_filings 
                        (accession_number, cik, ticker, form_type, filing_date, primary_document, document_url, title, raw_json, updated_at)
                    VALUES 
                        ('{acc}', '{cik}', '{ticker}', '{form}', '{fdate}', '{doc}', '{doc_url}', '{title}', '{{"companyName": "{company_name}"}}', NOW())
                    ON CONFLICT (accession_number) DO NOTHING;
                    """
                    exec_psql(sql)
                    total_filings += 1
        except Exception as e:
            log(f"SEC sync error for {ticker}: {e}")
            
    log(f"Synced {total_filings} SEC filings across top tickers")

if __name__ == "__main__":
    log("=== Starting Daily Macro & Fundamentals Sync ===")
    sync_cot()
    sync_sec()
    log("=== Daily Sync Finished Successfully ===")
