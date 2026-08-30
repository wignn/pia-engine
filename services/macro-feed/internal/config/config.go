package config

import (
	"time"

	"github.com/kelseyhightower/envconfig"
)

type Config struct {
	BondURL        string        `envconfig:"BOND_URL" default:"https://tradingeconomics.com/united-states/government-bond-yield"`
	TEApiKey       string        `envconfig:"TE_API_KEY"`
	BondSubject    string        `envconfig:"BOND_SUBJECT" default:"macro.feed.v1.bonds.us"`
	PollInterval   time.Duration `envconfig:"POLL_INTERVAL" default:"1m"`
	RequestTimeout time.Duration `envconfig:"REQUEST_TIMEOUT" default:"30s"`

	FredAPIKey   string        `envconfig:"FRED_API_KEY"`
	FredSubject  string        `envconfig:"FRED_SUBJECT" default:"macro.feed.v1.rates.us"`
	FredInterval time.Duration `envconfig:"FRED_INTERVAL" default:"1h"`

	NatsURL    string `envconfig:"NATS_URL" default:"nats://localhost:4222"`
	StreamName string `envconfig:"STREAM_NAME" default:"ATLSD_MACRO"`
}

func (c *Config) HasFred() bool {
	return c.FredAPIKey != ""
}

func Load() (*Config, error) {
	var cfg Config
	if err := envconfig.Process("", &cfg); err != nil {
		return nil, err
	}
	return &cfg, nil
}
