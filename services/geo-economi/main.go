package main

import (
	"context"
	"encoding/json"
	"errors"
	"log"
	"net/http"
	"os"
	"os/signal"
	"strconv"
	"strings"
	"syscall"
	"time"
)

func main() {
	store := NewSignalStore()
	store.SeedChokepoints(time.Now().UTC())
	macroStore := NewMacroStore()
	client := &http.Client{Timeout: 15 * time.Second}
	geoSources := []Source{
		GDELTSource{HTTPSource: HTTPSource{Client: client}},
	}
	if strings.EqualFold(os.Getenv("RELIEFWEB_ENABLED"), "true") {
		geoSources = append(geoSources, ReliefWebSource{HTTPSource: HTTPSource{Client: client}, AppName: envString("RELIEFWEB_APPNAME", "geosignals-mvp")})
	}
	refreshGeo := func() {
		ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
		defer cancel()
		for _, source := range geoSources {
			signals, err := source.Fetch(ctx)
			if err != nil {
				log.Printf("source=%s refresh failed: %v", source.Name(), err)
				continue
			}
			store.UpsertBatch(signals)
			log.Printf("source=%s signals=%d", source.Name(), len(signals))
		}
	}
	macroSources := make([]WorldBankMacroSource, 0, 5)
	for _, indicator := range []string{IndicatorInflation, IndicatorInterestRate, IndicatorGDPGrowth, IndicatorUnemployment, IndicatorDebtToGDP} {
		macroSources = append(macroSources, WorldBankMacroSource{HTTPSource: HTTPSource{Client: client}, Indicator: indicator})
	}
	refreshMacro := func() {
		ctx, cancel := context.WithTimeout(context.Background(), 90*time.Second)
		defer cancel()
		for _, source := range macroSources {
			points, err := source.Fetch(ctx)
			if err != nil {
				log.Printf("source=%s refresh failed: %v", source.Name(), err)
				continue
			}
			macroStore.UpsertBatch(points)
			log.Printf("source=%s points=%d", source.Name(), len(points))
		}
	}
	refreshGeo()
	go refreshMacro()

	mux := http.NewServeMux()
	mux.HandleFunc("GET /healthz", healthHandler)
	mux.HandleFunc("GET /api/v1/geosignals/map", mapHandler(store))
	mux.HandleFunc("GET /api/v1/macro/map", macroMapHandler(macroStore))

	server := &http.Server{Addr: envString("PORT", ":8080"), Handler: mux}
	if !strings.Contains(server.Addr, ":") {
		server.Addr = ":" + server.Addr
	}
	interval := time.Duration(envInt("REFRESH_INTERVAL_SECONDS", 600)) * time.Second
	macroInterval := time.Duration(envInt("MACRO_REFRESH_INTERVAL_SECONDS", 21600)) * time.Second
	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer stop()
	go func() {
		ticker := time.NewTicker(interval)
		defer ticker.Stop()
		for {
			select {
			case <-ticker.C:
				refreshGeo()
			case <-ctx.Done():
				return
			}
		}
	}()
	go func() {
		ticker := time.NewTicker(macroInterval)
		defer ticker.Stop()
		for {
			select {
			case <-ticker.C:
				refreshMacro()
			case <-ctx.Done():
				return
			}
		}
	}()
	go func() {
		<-ctx.Done()
		shutdownCtx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		_ = server.Shutdown(shutdownCtx)
	}()

	log.Printf("GeoSignals API listening on %s", server.Addr)
	if err := server.ListenAndServe(); err != nil && !errors.Is(err, http.ErrServerClosed) {
		log.Fatal(err)
	}
}

func healthHandler(w http.ResponseWriter, _ *http.Request) {
	writeJSON(w, http.StatusOK, map[string]string{"status": "ok"})
}

func macroMapHandler(store *MacroStore) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		query := r.URL.Query()
		indicator := strings.ToLower(strings.TrimSpace(query.Get("indicator")))
		if indicator == "" {
			indicator = IndicatorInflation
		}
		if !validIndicator(indicator) {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": "invalid indicator"})
			return
		}
		limit := parseInt(query.Get("limit"), 250)
		if limit < 1 || limit > 250 {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": "limit must be between 1 and 250"})
			return
		}
		w.Header().Set("Cache-Control", "public, max-age=300")
		writeJSON(w, http.StatusOK, store.Snapshot(MacroFilter{Indicator: indicator, Region: strings.TrimSpace(query.Get("region")), Limit: limit}, time.Now().UTC()))
	}
}

func mapHandler(store *SignalStore) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		query := r.URL.Query()
		filter := SignalFilter{
			Category:    strings.TrimSpace(query.Get("category")),
			Region:      strings.TrimSpace(query.Get("region")),
			MinSeverity: parseFloat(query.Get("min_severity")),
			Limit:       parseInt(query.Get("limit"), 100),
		}
		if from := parseTimeParam(query.Get("from")); !from.IsZero() {
			filter.From = from
		}
		if to := parseTimeParam(query.Get("to")); !to.IsZero() {
			filter.To = to
		}
		if filter.MinSeverity < 0 || filter.MinSeverity > 1 || filter.Limit < 1 || filter.Limit > 250 {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": "invalid query: min_severity must be 0..1 and limit 1..250"})
			return
		}
		writeJSON(w, http.StatusOK, store.Snapshot(filter, time.Now().UTC()))
	}
}

func writeJSON(w http.ResponseWriter, status int, value any) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(value)
}

func envString(name, fallback string) string {
	if value := strings.TrimSpace(os.Getenv(name)); value != "" {
		return value
	}
	return fallback
}

func envInt(name string, fallback int) int {
	value, err := strconv.Atoi(os.Getenv(name))
	if err != nil || value < 1 {
		return fallback
	}
	return value
}

func parseFloat(value string) float64 {
	if value == "" {
		return 0
	}
	parsed, err := strconv.ParseFloat(value, 64)
	if err != nil {
		return -1
	}
	return parsed
}

func parseInt(value string, fallback int) int {
	if value == "" {
		return fallback
	}
	parsed, err := strconv.Atoi(value)
	if err != nil {
		return 0
	}
	return parsed
}

func parseTimeParam(value string) time.Time {
	if value == "" {
		return time.Time{}
	}
	parsed, err := time.Parse(time.RFC3339, value)
	if err != nil {
		return time.Time{}
	}
	return parsed.UTC()
}
