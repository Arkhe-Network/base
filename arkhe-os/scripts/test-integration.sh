#!/bin/bash
# scripts/test-integration.sh
# Valida a integração dos novos crates no workspace

set -euo pipefail

echo "════════════════════════════════════════════"
echo "  ARKHE OS — Teste de Integração"
echo "════════════════════════════════════════════"

# 1. Compilação
echo "▶️  Compilando workspace..."
cargo build --workspace || { echo "❌ Build falhou"; exit 1; }

# 2. Testes unitários de todos os crates
echo "▶️  Executando testes..."
cargo test --workspace -- --nocapture || { echo "❌ Testes falharam"; exit 1; }

# 3. Testes específicos dos novos crates
echo "▶️  Testes específicos: GDID + Octonions"
cargo test -p arkhe-gdid || { echo "❌ Testes do GDID falharam"; exit 1; }
cargo test -p arkhe-octonions || { echo "❌ Testes dos Octonions falharam"; exit 1; }

echo "════════════════════════════════════════════"
echo "  ✅ Integração validada com sucesso"
echo "════════════════════════════════════════════"
