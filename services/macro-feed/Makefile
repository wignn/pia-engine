.PHONY: build run test tidy clean

APP_NAME := macro-feed
CMD_PATH := ./cmd/main.go

build:
	go build -o bin/$(APP_NAME) $(CMD_PATH)

run:
	go run $(CMD_PATH)

test:
	go test -v -race ./...

tidy:
	go mod tidy
	go mod verify

clean:
	rm -rf bin/
