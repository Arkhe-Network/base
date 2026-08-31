# dnssec_monitor_v15.py — Monitoramento DNSSEC automático

import dns.resolver
import dns.dnssec
import dns.rdatatype
import json
import time
import threading
from typing import Dict, List, Optional
from datetime import datetime, timedelta
import logging

class DNSSECMonitor:
    """
    Monitoramento automático de DNSSEC.
    Baseado em: dns-security-mcp e DNSSEC Validator API

    Verifica:
    - DS delegation records no pai
    - DNSKEY presence com KSK e ZSK classification
    - RRSIG coverage em record types comuns
    - AD bit support
    """

    def __init__(self, domain: str, check_interval: int = 3600):
        self.domain = domain
        self.check_interval = check_interval
        self.resolver = dns.resolver.Resolver()
        self.resolver.nameservers = ["1.1.1.1", "8.8.8.8"]
        self.resolver.use_edns(0, 4096, True)
        self._running = False
        self._thread = None
        self._last_result = None
        self._alerts = []

    def start(self):
        """Inicia monitoramento periódico"""
        if self._running:
            return

        self._running = True
        self._thread = threading.Thread(target=self._monitor_loop, daemon=True)
        self._thread.start()
        logging.info(f"DNSSEC Monitor started for {self.domain}")

    def stop(self):
        self._running = False
        if self._thread:
            self._thread.join(timeout=5)

    def _monitor_loop(self):
        """Loop de monitoramento"""
        while self._running:
            try:
                result = self.validate()
                self._last_result = result
                self._check_alerts(result)
                logging.info(f"DNSSEC validation for {self.domain}: score={result.get('score', 0)}")
            except Exception as e:
                logging.error(f"DNSSEC validation error: {e}")
            time.sleep(self.check_interval)

    def validate(self) -> Dict:
        """
        Valida a cadeia DNSSEC completa.
        Retorna score, grade e recomendações.
        """
        result = {
            "domain": self.domain,
            "checkedAt": datetime.utcnow().isoformat(),
            "dsAtParent": self._check_ds(),
            "dnskey": self._check_dnskey(),
            "rrsigCoverage": self._check_rrsig(),
            "adBitCheck": self._check_ad_bit(),
            "score": 0,
            "grade": "F",
            "issues": [],
            "recommendations": []
        }

        # Calcula score
        score = 0

        # DS check (30 pontos)
        if result["dsAtParent"]["found"]:
            score += 30

        # DNSKEY check (30 pontos)
        if result["dnskey"]["found"]:
            score += 30
            if result["dnskey"]["keySigningKeys"]:
                score += 10

        # RRSIG coverage (20 pontos)
        signed = result["rrsigCoverage"]["signed"]
        total = len(result["rrsigCoverage"]["checked"])
        if total > 0:
            score += int(20 * len(signed) / total)

        # AD bit (10 pontos)
        if result["adBitCheck"]["supported"]:
            score += 10

        result["score"] = min(100, score)

        # Grade
        if score >= 90:
            result["grade"] = "A"
        elif score >= 80:
            result["grade"] = "B"
        elif score >= 70:
            result["grade"] = "C"
        elif score >= 60:
            result["grade"] = "D"
        else:
            result["grade"] = "F"

        # Issues e recomendações
        if not result["dsAtParent"]["found"]:
            result["issues"].append("DS records not found at parent zone")
            result["recommendations"].append("Configure DS records at registrar")
        if not result["dnskey"]["found"]:
            result["issues"].append("DNSKEY records not found")
            result["recommendations"].append("Enable DNSSEC signing")
        if not result["adBitCheck"]["supported"]:
            result["issues"].append("AD bit not supported by resolver")
            result["recommendations"].append("Use DNSSEC-aware resolver")

        return result

    def _check_ds(self) -> Dict:
        """Verifica DS records no pai"""
        try:
            answers = self.resolver.resolve(self.domain, dns.rdatatype.DS, want_dnssec=True)
            return {
                "found": True,
                "parentDomain": self.domain,
                "recordCount": len(answers),
                "records": [
                    {
                        "keyTag": str(answer.key_tag),
                        "algorithm": str(answer.algorithm),
                        "digestType": str(answer.digest_type),
                        "digest": answer.digest.hex()[:20] + "..."
                    }
                    for answer in answers
                ],
                "issues": []
            }
        except Exception as e:
            return {"found": False, "parentDomain": self.domain, "recordCount": 0,
                    "records": [], "issues": [str(e)]}

    def _check_dnskey(self) -> Dict:
        """Verifica DNSKEY com KSK/ZSK classification"""
        try:
            answers = self.resolver.resolve(self.domain, dns.rdatatype.DNSKEY, want_dnssec=True)
            ksk = []
            zsk = []
            for answer in answers:
                if answer.flags & 0x100:  # KSK (flags 257)
                    ksk.append({"keyTag": str(answer.key_tag), "algorithm": str(answer.algorithm)})
                elif answer.flags & 0x80:  # ZSK (flags 256)
                    zsk.append({"keyTag": str(answer.key_tag), "algorithm": str(answer.algorithm)})

            return {
                "found": True,
                "keyCount": len(answers),
                "keySigningKeys": ksk,
                "zoneSigningKeys": zsk,
                "issues": []
            }
        except Exception as e:
            return {"found": False, "keyCount": 0, "keySigningKeys": [],
                    "zoneSigningKeys": [], "issues": [str(e)]}

    def _check_rrsig(self) -> Dict:
        """Verifica RRSIG coverage"""
        record_types = ["A", "AAAA", "MX", "TXT", "DNSKEY"]
        checked = []
        signed = []
        unsigned = []
        details = []

        for rtype in record_types:
            try:
                answers = self.resolver.resolve(self.domain, rtype, want_dnssec=True)
                checked.append(rtype)
                # Verifica se tem RRSIG
                if answers.response.answer:
                    has_rrsig = any(rr.rdtype == dns.rdatatype.RRSIG for rr in answers.response.answer)
                    if has_rrsig:
                        signed.append(rtype)
                    else:
                        unsigned.append(rtype)
                details.append({"type": rtype, "hasAnswer": True, "signed": has_rrsig})
            except:
                details.append({"type": rtype, "hasAnswer": False, "signed": False})

        return {
            "checked": checked,
            "signed": signed,
            "unsigned": unsigned,
            "details": details,
            "issues": []
        }

    def _check_ad_bit(self) -> Dict:
        """Verifica suporte ao AD bit"""
        try:
            answers = self.resolver.resolve(self.domain, "A", want_dnssec=True)
            ad_bit = "ad" in str(answers.response.flags)
            return {"supported": ad_bit, "issues": []}
        except:
            return {"supported": False, "issues": ["AD bit check failed"]}

    def _check_alerts(self, result: Dict):
        """Gera alertas baseados no resultado"""
        if result["score"] < 70:
            self._alerts.append({
                "timestamp": datetime.utcnow().isoformat(),
                "severity": "warning" if result["score"] >= 50 else "critical",
                "domain": self.domain,
                "score": result["score"],
                "grade": result["grade"],
                "issues": result["issues"]
            })
            logging.warning(f"DNSSEC alert for {self.domain}: score={result['score']}, grade={result['grade']}")

    def get_status(self) -> Dict:
        """Obtém status atual do monitoramento"""
        return {
            "domain": self.domain,
            "running": self._running,
            "last_check": self._last_result,
            "alerts": self._alerts[-10:],  # Últimos 10 alertas
            "alert_count": len(self._alerts)
        }


# ============================================================================
# API para Monitoramento
# ============================================================================

from flask import Flask, jsonify, request

app = Flask(__name__)
monitors = {}

@app.route('/api/dnssec/monitor', methods=['POST'])
def start_monitor():
    """Inicia monitoramento DNSSEC para um domínio"""
    data = request.json
    domain = data.get('domain')
    interval = data.get('interval', 3600)

    if domain in monitors:
        return jsonify({"error": "Domain already monitored"}), 400

    monitor = DNSSECMonitor(domain, interval)
    monitor.start()
    monitors[domain] = monitor

    return jsonify({"status": "started", "domain": domain, "interval": interval})

@app.route('/api/dnssec/status/<domain>', methods=['GET'])
def get_dnssec_status(domain):
    """Obtém status DNSSEC de um domínio"""
    if domain not in monitors:
        return jsonify({"error": "Domain not monitored"}), 404

    return jsonify(monitors[domain].get_status())

@app.route('/api/dnssec/validate', methods=['POST'])
def validate_dnssec():
    """Valida DNSSEC sob demanda"""
    data = request.json
    domain = data.get('domain')

    monitor = DNSSECMonitor(domain)
    result = monitor.validate()

    return jsonify(result)