#!/bin/bash
# activate_cloudflare_dnssec.sh

DOMAIN="catedral.os"
CLOUDFLARE_API_KEY="${CLOUDFLARE_API_KEY}"
CLOUDFLARE_EMAIL="${CLOUDFLARE_EMAIL}"
ZONE_ID=$(curl -s -X GET "https://api.cloudflare.com/client/v4/zones?name=$DOMAIN" \
    -H "X-Auth-Email: $CLOUDFLARE_EMAIL" \
    -H "X-Auth-Key: $CLOUDFLARE_API_KEY" \
    -H "Content-Type: application/json" | jq -r '.result[0].id')

# 1. Ativar DNSSEC
curl -X PATCH "https://api.cloudflare.com/client/v4/zones/$ZONE_ID/dnssec" \
    -H "X-Auth-Email: $CLOUDFLARE_EMAIL" \
    -H "X-Auth-Key: $CLOUDFLARE_API_KEY" \
    -H "Content-Type: application/json" \
    --data '{"status": "active"}'

# 2. Obter os DS records
DS_RECORDS=$(curl -s -X GET "https://api.cloudflare.com/client/v4/zones/$ZONE_ID/dnssec" \
    -H "X-Auth-Email: $CLOUDFLARE_EMAIL" \
    -H "X-Auth-Key: $CLOUDFLARE_API_KEY" \
    -H "Content-Type: application/json" | jq '.result')

echo "✅ DNSSEC ativado para $DOMAIN"
echo "DS Records a serem adicionados ao registrador:"
echo "$DS_RECORDS"