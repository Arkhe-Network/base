#!/bin/bash
# verify_dnssec_chain.sh

DOMAIN="catedral.os"

echo "🔍 Verificando cadeia de confiança DNSSEC para $DOMAIN"

# 1. Verificar DNSKEY
echo "1. DNSKEY:"
dig +dnssec $DOMAIN DNSKEY | grep -E "DNSKEY|RRSIG"

# 2. Verificar DS no pai
echo "2. DS record no pai:"
dig +dnssec $DOMAIN DS | grep -E "DS|RRSIG"

# 3. Verificar cadeia completa
echo "3. Cadeia de confiança:"
dig +sigchase $DOMAIN SOA

# 4. Verificar AD bit
echo "4. AD bit:"
dig +dnssec $DOMAIN A | grep -E "flags:.*ad"

# 5. Verificar _did TXT com DNSSEC
echo "5. _did TXT:"
dig +dnssec _did.$DOMAIN TXT

echo "✅ Verificação DNSSEC concluída"