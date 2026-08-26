package scraper

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"os"
	"regexp"
	"strconv"
	"strings"
	"time"

	"macro-feed/internal/model"

	"github.com/PuerkitoBio/goquery"
	"github.com/gocolly/colly/v2"
)

var (
	forecastPattern = regexp.MustCompile(`TEForecast\s*=\s*\[([^\]]*)\]`)
	numberPattern   = regexp.MustCompile(`[-+]?\d+(?:[.,]\d+)?`)
)

type BondScraper struct {
	sourceURL  string
	apiKey     string
	httpClient *http.Client
}

func NewBondScraper(sourceURL, apiKey string, timeout time.Duration) *BondScraper {
	if timeout <= 0 {
		timeout = 30 * time.Second
	}

	return &BondScraper{
		sourceURL:  sourceURL,
		apiKey:     apiKey,
		httpClient: &http.Client{Timeout: timeout},
	}
}

func (s *BondScraper) Fetch(ctx context.Context) (model.DashboardData, error) {
	var data model.DashboardData
	var scrapeErr error

	c := colly.NewCollector(
		colly.AllowedDomains("tradingeconomics.com", "www.tradingeconomics.com"),
	)
	c.UserAgent = "Mozilla/5.0 (compatible; macro-feed/1.0)"

	c.OnHTML("html", func(e *colly.HTMLElement) {
		data = s.parseDashboard(e.DOM)
	})

	c.OnError(func(r *colly.Response, err error) {
		scrapeErr = fmt.Errorf("fetch source: %w", err)
	})

	if err := c.Visit(s.sourceURL); err != nil {
		return model.DashboardData{}, err
	}
	if scrapeErr != nil {
		return model.DashboardData{}, scrapeErr
	}
	if len(data.Bonds) == 0 {
		return model.DashboardData{}, errors.New("source page contained no bond rows")
	}

	historyTemplate := os.Getenv("TE_HISTORY_URL")
	data.Histories = map[string][]model.HistoryPoint{}
	if historyTemplate != "" {
		data.History, data.HistoryMessage = s.fetchHistory(ctx, historyTemplate, "USGG10YR:IND")
		if len(data.History) > 0 {
			data.Histories["USGG10YR:IND"] = data.History
		}
		if strings.Contains(historyTemplate, "{symbol}") {
			for _, bond := range data.Bonds {
				if bond.Symbol == "USGG10YR:IND" {
					continue
				}
				points, _ := s.fetchHistory(ctx, historyTemplate, bond.Symbol)
				if len(points) > 0 {
					data.Histories[bond.Symbol] = points
				}
			}
		}
	}

	// Fallback calculation from delta yield
	if len(data.Histories) == 0 {
		for _, bond := range data.Bonds {
			derived := generateDerivedHistory(bond.Yield, bond.DayChange, bond.MonthChange, bond.YearChange)
			if len(derived) > 0 {
				data.Histories[bond.Symbol] = derived
			}
		}
		if tenYr, ok := data.Histories["USGG10YR:IND"]; ok {
			data.History = tenYr
		} else if len(data.Bonds) > 0 {
			data.History = data.Histories[data.Bonds[0].Symbol]
		}
		data.HistoryMessage = "Riwayat dihitung otomatis dari delta perubahan yield (1 Thn, 1 Bln, 1 Hari lalu)."
	}

	data.HistoryAvailable = len(data.Histories) > 0
	data.FetchedAt = time.Now().UTC().Format(time.RFC3339)
	return data, nil
}

func (s *BondScraper) parseDashboard(doc *goquery.Selection) model.DashboardData {
	data := model.DashboardData{
		Source:  s.sourceURL,
		History: []model.HistoryPoint{},
	}

	data.Quote.Actual = parseNumber(doc.Find("#market_stats_grid #market_last").First().Text())
	data.Quote.DailyChange = parseNumber(doc.Find("#market_stats_grid #market_daily_chg").First().Text())
	data.Quote.DailyPercent = parseNumber(doc.Find("#market_stats_grid #market_daily_Pchg").First().Text())

	doc.Find("#market_stats_grid .market-header-value").Each(func(_ int, sel *goquery.Selection) {
		label := strings.ToLower(strings.TrimSpace(sel.Find(".te-market-header").First().Text()))
		value := parseNumber(sel.Find("span").Last().Text())
		switch {
		case strings.Contains(label, "monthly"):
			data.Quote.Monthly = value
		case strings.Contains(label, "yearly"):
			data.Quote.Yearly = value
		case strings.Contains(label, "forecast"):
			data.Quote.Forecast = value
		}
	})

	if data.Quote.DailyPercent == 0 && data.Quote.Actual != 0 {
		data.Quote.DailyPercent = data.Quote.DailyChange / data.Quote.Actual * 100
	}

	doc.Find("table.table-heatmap tr[data-symbol]").Each(func(_ int, row *goquery.Selection) {
		cells := row.Find("td")
		if cells.Length() < 7 {
			return
		}
		data.Bonds = append(data.Bonds, model.Bond{
			Symbol:      strings.TrimSpace(row.AttrOr("data-symbol", "")),
			Name:        cleanText(cells.Eq(0).Find("a").Text()),
			Yield:       parseNumber(cells.Eq(1).Text()),
			DayChange:   parseNumber(cells.Eq(3).Text()),
			MonthChange: parseNumber(cells.Eq(4).Text()),
			YearChange:  parseNumber(cells.Eq(5).Text()),
			Date:        cleanText(cells.Eq(6).Text()),
		})
	})

	doc.Find("table").Each(func(_ int, table *goquery.Selection) {
		if !strings.EqualFold(cleanText(table.Find("thead th").First().Text()), "related") {
			return
		}
		table.Find("tr").Each(func(_ int, row *goquery.Selection) {
			cells := row.Find("td")
			if cells.Length() < 5 {
				return
			}
			data.Related = append(data.Related, model.Related{
				Name:     cleanText(cells.Eq(0).Find("a").Text()),
				Last:     parseNumber(cells.Eq(1).Text()),
				Previous: parseNumber(cells.Eq(2).Text()),
				Unit:     cleanText(cells.Eq(3).Text()),
				Date:     cleanText(cells.Eq(4).Text()),
			})
		})
	})

	doc.Find("script").Each(func(_ int, script *goquery.Selection) {
		text := script.Text()
		matches := forecastPattern.FindStringSubmatch(text)
		if len(matches) == 2 {
			data.Forecast = parseForecastNumbers(matches[1])
		}
	})

	return data
}

func (s *BondScraper) fetchHistory(ctx context.Context, template, symbol string) ([]model.HistoryPoint, string) {
	url := strings.TrimSpace(template)
	if url == "" {
		return []model.HistoryPoint{}, "Historical data is not configured."
	}
	url = strings.ReplaceAll(url, "{symbol}", symbol)
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, url, nil)
	if err != nil {
		return []model.HistoryPoint{}, "Historical data endpoint is invalid."
	}
	if s.apiKey != "" {
		req.Header.Set("Authorization", "Client "+s.apiKey)
	}
	resp, err := s.httpClient.Do(req)
	if err != nil {
		return []model.HistoryPoint{}, "Historical data could not be loaded."
	}
	defer resp.Body.Close()
	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		return []model.HistoryPoint{}, fmt.Sprintf("Historical data endpoint returned HTTP %d.", resp.StatusCode)
	}
	body, err := io.ReadAll(io.LimitReader(resp.Body, 8<<20))
	if err != nil {
		return []model.HistoryPoint{}, "Historical data response could not be read."
	}
	points := decodeHistory(body)
	if len(points) == 0 {
		return points, "Historical endpoint returned no points."
	}
	return points, ""
}

func generateDerivedHistory(currentYield, dayChg, monthChg, yearChg float64) []model.HistoryPoint {
	if currentYield == 0 {
		return []model.HistoryPoint{}
	}
	t := time.Now().UTC()
	return []model.HistoryPoint{
		{Date: t.AddDate(-1, 0, 0).Format("2006-01-02"), Value: currentYield - yearChg},
		{Date: t.AddDate(0, -1, 0).Format("2006-01-02"), Value: currentYield - monthChg},
		{Date: t.AddDate(0, 0, -1).Format("2006-01-02"), Value: currentYield - dayChg},
		{Date: t.Format("2006-01-02"), Value: currentYield},
	}
}

func parseNumber(text string) float64 {
	matches := numberPattern.FindString(strings.ReplaceAll(cleanText(text), ",", ""))
	if matches == "" {
		return 0
	}
	val, _ := strconv.ParseFloat(matches, 64)
	return val
}

func parseForecastNumbers(text string) []float64 {
	parts := strings.Split(text, ",")
	values := make([]float64, 0, len(parts))
	for _, part := range parts {
		clean := strings.TrimSpace(part)
		if clean == "" {
			continue
		}
		val := parseNumber(clean)
		if val != 0 {
			values = append(values, val)
		}
	}
	return values
}

func cleanText(text string) string {
	return strings.Join(strings.Fields(strings.ReplaceAll(text, " ", " ")), " ")
}

func decodeHistory(body []byte) []model.HistoryPoint {
	var payload any
	if json.Unmarshal(body, &payload) != nil {
		return nil
	}
	return historyFromValue(payload)
}

func historyFromValue(value any) []model.HistoryPoint {
	switch item := value.(type) {
	case []any:
		points := make([]model.HistoryPoint, 0, len(item))
		for _, child := range item {
			if point, ok := historyPoint(child); ok {
				points = append(points, point)
			}
		}
		return points
	case map[string]any:
		for _, key := range []string{"data", "results", "history", "values"} {
			if child, ok := item[key]; ok {
				if points := historyFromValue(child); len(points) > 0 {
					return points
				}
			}
		}
	}
	return nil
}

func historyPoint(value any) (model.HistoryPoint, bool) {
	item, ok := value.(map[string]any)
	if !ok {
		return model.HistoryPoint{}, false
	}
	date := firstString(item, "date", "Date", "datetime", "Datetime")
	valueNumber, ok := firstNumber(item, "value", "Value", "close", "Close")
	if date == "" || !ok {
		return model.HistoryPoint{}, false
	}
	return model.HistoryPoint{Date: date, Value: valueNumber}, true
}

func firstString(item map[string]any, keys ...string) string {
	for _, key := range keys {
		if val, ok := item[key].(string); ok && strings.TrimSpace(val) != "" {
			return strings.TrimSpace(val)
		}
	}
	return ""
}

func firstNumber(item map[string]any, keys ...string) (float64, bool) {
	for _, key := range keys {
		switch val := item[key].(type) {
		case float64:
			return val, true
		case json.Number:
			parsed, err := val.Float64()
			if err == nil {
				return parsed, true
			}
		}
	}
	return 0, false
}
