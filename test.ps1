$ApiKey = "wi_live_907c43a29c2ff84c8a5747d5ebfbc7bc37e313e41b32d8c3"
$Headers = @{ "X-API-Key" = $ApiKey }

Invoke-RestMethod `
  -Method GET `
  -Uri "http://localhost:8090/api/v1/forex/news/latest?limit=5" `
  -Headers $Headers | ConvertTo-Json -Depth 8