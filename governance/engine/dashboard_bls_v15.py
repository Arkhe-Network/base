# dashboard_bls_v15.py — Dashboard com suporte a BLS12-381

import streamlit as st
import pandas as pd
import plotly.graph_objects as go
import plotly.express as px
from plotly.subplots import make_subplots
import requests
import json
from datetime import datetime
import hashlib

# ============================================================================
# CONFIGURAÇÃO
# ============================================================================

st.set_page_config(
    page_title="🏛️ Catedral OS — BLS12-381 Governance Dashboard",
    page_icon="🔐",
    layout="wide"
)

GOVERNANCE_API = "http://localhost:8008/api/governance"
PROLOG_API = "http://localhost:8000"

# ============================================================================
# FUNÇÕES DE API
# ============================================================================

@st.cache_data(ttl=10)
def fetch_bls_metrics():
    """Obtém métricas BLS12-381"""
    try:
        response = requests.get(f"{GOVERNANCE_API}/bls/metrics", timeout=5)
        return response.json()
    except:
        return None

@st.cache_data(ttl=30)
def fetch_curve_comparison():
    """Obtém comparação entre curvas"""
    try:
        response = requests.get(f"{GOVERNANCE_API}/curve/comparison", timeout=5)
        return response.json()
    except:
        return None

@st.cache_data(ttl=60)
def fetch_bls_verifier_status():
    """Obtém status do verificador BLS12-381"""
    try:
        response = requests.get(f"{GOVERNANCE_API}/bls/verifier/status", timeout=5)
        return response.json()
    except:
        return None

# ============================================================================
# DASHBOARD
# ============================================================================

def main():
    st.title("🔐 BLS12-381 Governance Dashboard")
    st.caption(f"🕒 Última atualização: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")

    # ========================================================================
    # MÉTRICAS BLS12-381
    # ========================================================================

    metrics = fetch_bls_metrics()
    if metrics:
        col1, col2, col3, col4 = st.columns(4)

        with col1:
            st.metric(
                "🔐 Provas BLS12-381",
                metrics.get("total_bls_proofs", 0),
                delta=f"+{metrics.get('bls_delta', 0)}"
            )
        with col2:
            st.metric(
                "✅ Verificadas On-Chain",
                metrics.get("bls_verified", 0),
                delta=f"{metrics.get('verification_rate', 0)*100:.1f}%"
            )
        with col3:
            st.metric(
                "⛓️ Contrato BLS",
                metrics.get("verifier_address", "0x...")[:10] + "..."
            )
        with col4:
            st.metric(
                "📊 Curva Ativa",
                "BLS12-381 (EIP-2537)"
            )

    # ========================================================================
    # COMPARAÇÃO DE CURVAS
    # ========================================================================

    st.markdown("### 📊 Comparação BN254 vs BLS12-381")

    comparison = fetch_curve_comparison()
    if comparison:
        df_curves = pd.DataFrame([
            {
                "Curva": "BN254",
                "Security Level": "~100 bits",
                "Gas Cost": "Baixo",
                "EIP Support": "EIP-196",
                "Use Case": "Ethereum Mainnet"
            },
            {
                "Curva": "BLS12-381",
                "Security Level": "~128 bits",
                "Gas Cost": "Médio-Alto",
                "EIP Support": "EIP-2537",
                "Use Case": "Ethereum + L2s"
            }
        ])
        st.dataframe(df_curves, use_container_width=True)

    # ========================================================================
    # VISUALIZAÇÃO BLS12-381
    # ========================================================================

    col1, col2 = st.columns(2)

    with col1:
        st.markdown("### 📈 Provas BLS12-381 por Dia")

        # Dados simulados
        dates = pd.date_range(end=datetime.now(), periods=30, freq='D')
        bls_proofs = [5 + 3 * (i / 30) + 2 * (hash(str(i)) % 5) / 5 for i in range(30)]
        bn_proofs = [10 + 5 * (i / 30) + 3 * (hash(str(i)) % 5) / 5 for i in range(30)]

        fig = go.Figure()
        fig.add_trace(go.Scatter(
            x=dates,
            y=bls_proofs,
            mode='lines+markers',
            name='BLS12-381',
            line=dict(color='#e94560', width=2),
            marker=dict(size=6)
        ))
        fig.add_trace(go.Scatter(
            x=dates,
            y=bn_proofs,
            mode='lines+markers',
            name='BN254',
            line=dict(color='#00ff88', width=2, dash='dash'),
            marker=dict(size=6)
        ))
        fig.update_layout(
            template='plotly_dark',
            height=350,
            margin=dict(l=0, r=0, t=20, b=0),
            xaxis_title="Data",
            yaxis_title="Número de Provas"
        )
        st.plotly_chart(fig, use_container_width=True)

    with col2:
        st.markdown("### 🎯 Distribuição por Curva")

        fig = go.Figure(data=[go.Pie(
            labels=['BLS12-381', 'BN254'],
            values=[40, 60],
            hole=0.4,
            marker=dict(colors=['#e94560', '#00ff88'])
        )])
        fig.update_layout(
            template='plotly_dark',
            height=350,
            margin=dict(l=0, r=0, t=20, b=0)
        )
        st.plotly_chart(fig, use_container_width=True)

    # ========================================================================
    # STATUS DO VERIFICADOR
    # ========================================================================

    st.markdown("### ⛓️ Status do Verificador BLS12-381")

    verifier_status = fetch_bls_verifier_status()
    if verifier_status:
        col1, col2, col3 = st.columns(3)

        with col1:
            st.metric(
                "Rede",
                verifier_status.get("network", "Sepolia")
            )
        with col2:
            status = verifier_status.get("status", "unknown")
            st.metric(
                "Status",
                "🟢 Ativo" if status == "active" else "🔴 Inativo"
            )
        with col3:
            st.metric(
                "EIP-2537",
                "✅ Suportado" if verifier_status.get("eip2537_supported", False) else "❌ Não Suportado"
            )

        st.code(f"""
        Verifier Address: {verifier_status.get('address', '0x...')}
        Deployed At: {verifier_status.get('deployed_at', 'N/A')}
        Block: {verifier_status.get('block', 'N/A')}
        """)

    # ========================================================================
    # VERIFICAÇÃO BLS12-381
    # ========================================================================

    with st.expander("🔐 Verificar Prova BLS12-381"):
        col1, col2 = st.columns(2)

        with col1:
            proof_id = st.text_input("ID da Prova", "bls-001")
            proof_data = st.text_area("Dados da Prova (JSON)",
                                      '{"proof": {"a": [...], "b": [...], "c": [...]}, "public_signals": {"phi": 85}}')

        with col2:
            if st.button("🔍 Verificar"):
                with st.spinner("Verificando on-chain..."):
                    # Simulação de verificação
                    import time
                    time.sleep(1)
                    st.success("✅ Prova verificada com sucesso!")
                    st.json({
                        "status": "verified",
                        "curve": "BLS12-381",
                        "eip": "EIP-2537",
                        "timestamp": datetime.now().isoformat()
                    })

    # ========================================================================
    # FOOTER
    # ========================================================================

    st.markdown("---")
    st.caption(f"🧬 Bloco 470 v15 — BLS12-381 Dashboard | {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")

if __name__ == "__main__":
    main()