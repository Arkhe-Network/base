#!/usr/bin/env python3
"""
ARKHE — Crate-level Compliance Scoring
Calcula score por crate baseado em:
  1. Compilabilidade (cargo check --crate)
  2. Cobertura de testes (cargo test --crate)
  3. Presença de stubs (heurística: busca por todo!() / unimplemented!())
  4. Documentação (presença de README ou docstrings)

Uso:
  python scripts/score-crates.py [--json] [--badge]

Saída:
  - Tabela no terminal
  - score-report.json (com --json)
  - score-badge.svg (com --badge)
"""

import json
import subprocess
import sys
import os
import re
import argparse
from pathlib import Path
from typing import Dict, List, Tuple

WORKSPACE_ROOT = Path(".")


def run(cmd: List[str], cwd=None) -> Tuple[int, str, str]:
    """Executa comando e retorna (returncode, stdout, stderr)."""
    result = subprocess.run(
        cmd, capture_output=True, text=True, cwd=cwd or WORKSPACE_ROOT
    )
    return result.returncode, result.stdout, result.stderr


def get_workspace_crates() -> List[str]:
    """Lista crates do workspace via cargo metadata."""
    rc, out, err = run(["cargo", "metadata", "--format-version", "1", "--no-deps"])
    if rc != 0:
        print(f"ERRO: cargo metadata falhou: {err}", file=sys.stderr)
        sys.exit(1)
    meta = json.loads(out)
    crates = []
    for member_id in meta.get("workspace_members", []):
        for pkg in meta.get("packages", []):
            if pkg["id"] == member_id:
                crates.append(pkg["name"])
    return sorted(crates)


def score_compilability(crate: str) -> Tuple[int, str]:
    """0-100 baseado em cargo check -p <crate>."""
    rc, _, err = run(["cargo", "check", "-p", crate])
    if rc == 0:
        return 100, "compila"
    # Contar erros aproximados
    errors = err.count("error[")
    if errors == 0:
        errors = err.lower().count("error")
    score = max(0, 100 - errors * 5)
    return score, f"{errors} erro(s)"


def score_tests(crate: str) -> Tuple[int, str]:
    """0-100 baseado em cargo test -p <crate>."""
    rc, out, err = run(["cargo", "test", "-p", crate, "--no-fail-fast"])
    combined = out + err
    # Procurar por "test result:"
    match = re.search(r"test result: (\w+)\. (\d+) passed; (\d+) failed", combined)
    if match:
        passed = int(match.group(2))
        failed = int(match.group(3))
        total = passed + failed
        if total == 0:
            return 0, "0 testes"
        score = int((passed / total) * 100)
        return score, f"{passed}/{total} passaram"
    if rc == 0 and "running 0 tests" in combined:
        return 0, "0 testes"
    if rc == 0:
        return 100, "todos passaram (sem testes explicitos)"
    return 0, "falha na execução de testes"


def score_stubs(crate: str) -> Tuple[int, str]:
    """0-100 — penaliza todo!(), unimplemented!(), stub, placeholder."""
    crate_path = WORKSPACE_ROOT / "crates" / crate.replace("-", "_")
    if not crate_path.exists():
        # Tentar encontrar diretório do crate
        for d in (WORKSPACE_ROOT / "crates").iterdir():
            if d.name.replace("-", "_") == crate.replace("-", "_"):
                crate_path = d
                break

    if not crate_path.exists():
        return 0, "diretório não encontrado"

    stub_patterns = [
        r"\btodo!\(\)",
        r"\bunimplemented!\(\)",
        r"\bpanic!\(\)",
        r"#\[cfg\(test\)\]\s*\n\s*fn\s+\w+\s*\(\)\s*\{\s*\}",  # teste vazio
        r"Ok\(vec!\[\]\)",  # retorno vazio stub
        r"allowed:\s*true",  # hardcoded allow
    ]

    total_stubs = 0
    total_lines = 0
    for root, _, files in os.walk(crate_path):
        for fname in files:
            if not fname.endswith(".rs"):
                continue
            fpath = Path(root) / fname
            try:
                content = fpath.read_text(encoding="utf-8")
            except Exception:
                continue
            lines = content.splitlines()
            total_lines += len(lines)
            for pattern in stub_patterns:
                total_stubs += len(re.findall(pattern, content))

    if total_lines == 0:
        return 0, "0 linhas"

    stub_density = total_stubs / total_lines
    # Penalidade: cada stub reduz ~10 pontos, saturando em 0
    score = max(0, int(100 - (stub_density * 1000)))
    return score, f"{total_stubs} stub(s) em {total_lines} linhas"


def score_documentation(crate: str) -> Tuple[int, str]:
    """0-100 — presença de README, lib.rs doc, exemplos."""
    crate_path = WORKSPACE_ROOT / "crates" / crate.replace("-", "_")
    if not crate_path.exists():
        for d in (WORKSPACE_ROOT / "crates").iterdir():
            if d.name.replace("-", "_") == crate.replace("-", "_"):
                crate_path = d
                break

    score = 0
    checks = []

    if (crate_path / "README.md").exists():
        score += 40
        checks.append("README")
    if (crate_path / "examples").exists() and any((crate_path / "examples").iterdir()):
        score += 30
        checks.append("examples")

    # Verificar docstrings em lib.rs ou main.rs
    for entry in ["src/lib.rs", "src/main.rs"]:
        f = crate_path / entry
        if f.exists():
            content = f.read_text(encoding="utf-8")
            if "//!" in content or "/*!" in content:
                score += 30
                checks.append("crate-doc")
                break

    return score, ", ".join(checks) if checks else "sem docs"


def calculate_composite(scores: Dict[str, Dict[str, int]]) -> Dict[str, float]:
    """Calcula score composto ponderado por crate."""
    weights = {
        "compilabilidade": 0.30,
        "testes": 0.25,
        "stubs": 0.25,
        "documentacao": 0.20,
    }
    result = {}
    for crate, data in scores.items():
        composite = sum(
            data.get(k, 0) * w for k, w in weights.items()
        )
        result[crate] = round(composite, 1)
    return result


def generate_badge(global_score: float, output_path: str = "score-badge.svg"):
    """Gera badge SVG com score global."""
    color = "brightgreen" if global_score >= 80 else "green" if global_score >= 60 else "yellow" if global_score >= 40 else "red"
    svg = f"""<svg xmlns="http://www.w3.org/2000/svg" width="120" height="20">
  <linearGradient id="a" x2="0" y2="100%%">
    <stop offset="0" stop-color="#bbb" stop-opacity=".1"/>
    <stop offset="1" stop-opacity=".1"/>
  </linearGradient>
  <rect rx="3" width="120" height="20" fill="#555"/>
  <rect rx="3" x="55" width="65" height="20" fill="{color}"/>
  <path fill="{color}" d="M55 0h4v20h-4z"/>
  <rect rx="3" width="120" height="20" fill="url(#a)"/>
  <g fill="#fff" text-anchor="middle" font-family="DejaVu Sans,Verdana,Geneva,sans-serif" font-size="11">
    <text x="27.5" y="15" fill="#010101" fill-opacity=".3">ARKHE</text>
    <text x="27.5" y="14">ARKHE</text>
    <text x="87.5" y="15" fill="#010101" fill-opacity=".3">{global_score:.0f}%%</text>
    <text x="87.5" y="14">{global_score:.0f}%%</text>
  </g>
</svg>"""
    Path(output_path).write_text(svg)
    print(f"Badge gerado: {output_path}")


def main():
    parser = argparse.ArgumentParser(description="ARKHE Crate Scoring")
    parser.add_argument("--json", action="store_true", help="Exportar score-report.json")
    parser.add_argument("--badge", action="store_true", help="Gerar score-badge.svg")
    args = parser.parse_args()

    print("=" * 70)
    print("ARKHE — Crate-Level Compliance Scoring")
    print("=" * 70)

    crates = get_workspace_crates()
    if not crates:
        print("ERRO: Nenhum crate encontrado no workspace.", file=sys.stderr)
        sys.exit(1)

    print(f"Crates detectados: {len(crates)}")
    print("-" * 70)

    scores: Dict[str, Dict[str, any]] = {}

    for crate in crates:
        print(f"\n📦 {crate}")
        c_score, c_note = score_compilability(crate)
        t_score, t_note = score_tests(crate)
        s_score, s_note = score_stubs(crate)
        d_score, d_note = score_documentation(crate)

        scores[crate] = {
            "compilabilidade": c_score,
            "compilabilidade_nota": c_note,
            "testes": t_score,
            "testes_nota": t_note,
            "stubs": s_score,
            "stubs_nota": s_note,
            "documentacao": d_score,
            "documentacao_nota": d_note,
        }

        print(f"   Compilabilidade : {c_score:3d}/100  ({c_note})")
        print(f"   Testes          : {t_score:3d}/100  ({t_note})")
        print(f"   Stubs           : {s_score:3d}/100  ({s_note})")
        print(f"   Documentação    : {d_score:3d}/100  ({d_note})")

    composites = calculate_composite(scores)
    global_score = round(sum(composites.values()) / len(composites), 1) if composites else 0.0

    print("\n" + "=" * 70)
    print("RESUMO")
    print("=" * 70)
    for crate in crates:
        print(f"{crate:30s} → {composites[crate]:5.1f}/100")
    print("-" * 70)
    print(f"{'SCORE GLOBAL':30s} → {global_score:5.1f}/100")
    print("=" * 70)

    if args.json:
        report = {
            "global_score": global_score,
            "crates": {crate: {**scores[crate], "composite": composites[crate]} for crate in crates},
            "methodology": {
                "weights": {"compilabilidade": 0.30, "testes": 0.25, "stubs": 0.25, "documentacao": 0.20},
                "timestamp": subprocess.run(["date", "-Iseconds"], capture_output=True, text=True).stdout.strip(),
            }
        }
        with open("score-report.json", "w") as f:
            json.dump(report, f, indent=2)
        print("\n📄 score-report.json gerado.")

    if args.badge:
        generate_badge(global_score)

    # Exit code: falha se global < 60 (threshold para merge)
    if global_score < 60:
        print(f"\n⚠️  Score global {global_score} abaixo do threshold (60).")
        sys.exit(1)
    else:
        print(f"\n✅ Score global {global_score} acima do threshold (60).")
        sys.exit(0)


if __name__ == "__main__":
    main()