# dashboard_v12.py — Streamlit com autenticação JWT e DIDs

import streamlit as st
import pandas as pd
import plotly.graph_objects as go
import plotly.express as px
import requests
import json
import jwt
import hashlib
import os
from datetime import datetime, timedelta
import time
from typing import Dict, Optional
from did_manager_v12 import DIDManagerV12
import secrets

# ============================================================================
# CONFIGURAÇÃO DE AUTENTICAÇÃO
# ============================================================================

# Carregar secret do ambiente ou gerar um aleatório em desenvolvimento
JWT_SECRET = os.environ.get('DASHBOARD_JWT_SECRET')
if not JWT_SECRET:
    # Apenas em desenvolvimento: gerar um secret fixo, mas avisar
    JWT_SECRET = secrets.token_urlsafe(32)  # 256 bits
    print("⚠️ WARNING: JWT_SECRET não definido. Usando secret aleatório temporário.")

DID_METHOD = os.environ.get("DID_METHOD", "key")

def create_jwt(did: str, role: str = "viewer") -> str:
    """Cria um JWT para autenticação baseada em DID."""
    payload = {
        "did": did,
        "role": role,
        "exp": datetime.utcnow() + timedelta(hours=1)
    }
    return jwt.encode(payload, JWT_SECRET, algorithm="HS256")

def verify_jwt(token: str) -> Optional[Dict]:
    """Verifica o JWT e retorna o payload."""
    try:
        return jwt.decode(token, JWT_SECRET, algorithms=["HS256"])
    except jwt.InvalidTokenError:
        return None

def fetch_metrics():
    return {
        "total_prs": 150,
        "high_coherence_rate": 0.85,
        "critical_prs": 5,
        "active_agents": 12,
        "merged_prs": 100,
        "vetoed_prs": 20
    }

# ============================================================================
# AUTENTICAÇÃO
# ============================================================================

def auth_ui():
    """Interface de autenticação com DID."""
    st.sidebar.markdown("### 🔑 Autenticação")

    # Input do DID
    did_input = st.sidebar.text_input("DID", placeholder="did:key:z6Mk...")
    if did_input:
        # Tenta verificar o DID
        did_doc = DIDManagerV12.resolve_did(did_input)
        if did_doc:
            st.sidebar.success("✅ DID verificado")
            # Gera token JWT
            token = create_jwt(did_input, "admin")
            st.sidebar.session_state['jwt_token'] = token
            st.sidebar.session_state['did'] = did_input
            st.rerun()
        else:
            st.sidebar.error("❌ DID não encontrado")

    # Se já autenticado
    if 'jwt_token' in st.sidebar.session_state:
        payload = verify_jwt(st.sidebar.session_state['jwt_token'])
        if payload:
            st.sidebar.success(f"✅ Conectado como {payload['did']}")
            if st.sidebar.button("Logout"):
                del st.sidebar.session_state['jwt_token']
                del st.sidebar.session_state['did']
                st.rerun()
            return True

    return False

# ============================================================================
# PÁGINAS DO DASHBOARD
# ============================================================================

def overview_page():
    """Página de visão geral com métricas e gráficos."""
    st.title("🏛️ Catedral OS — Governance Dashboard v12")
    st.caption(f"🕒 Última atualização: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")

    # Dados simulados (em produção, via API)
    metrics = fetch_metrics()
    if not metrics:
        st.warning("⚠️ Não foi possível obter métricas. Verifique a API.")
        return

    # Layout de métricas
    col1, col2, col3, col4 = st.columns(4)
    col1.metric("Total PRs", metrics.get("total_prs", 0))
    col2.metric("Coerência ≥ 85%", f"{metrics.get('high_coherence_rate', 0)*100:.1f}%")
    col3.metric("Críticos", metrics.get("critical_prs", 0))
    col4.metric("Agentes Ativos", metrics.get("active_agents", 0))

    # Gráficos
    col1, col2 = st.columns(2)
    with col1:
        fig_coherence = go.Figure()
        fig_coherence.add_trace(go.Scatter(
            x=pd.date_range(end=datetime.now(), periods=30, freq='D'),
            y=[0.7 + 0.3 * i/30 for i in range(30)],
            mode='lines+markers',
            name='Φ (Coerência)'
        ))
        fig_coherence.add_hline(y=0.85, line_dash="dash", line_color="green")
        fig_coherence.update_layout(template='plotly_dark', height=300)
        st.plotly_chart(fig_coherence, use_container_width=True)

    with col2:
        fig_pie = go.Figure(data=[go.Pie(
            labels=['Auto-Merge', 'Veto', 'Manual'],
            values=[metrics.get('merged_prs', 0), metrics.get('vetoed_prs', 0),
                    metrics.get('total_prs', 0) - metrics.get('merged_prs', 0) - metrics.get('vetoed_prs', 0)],
            hole=0.4
        )])
        fig_pie.update_layout(template='plotly_dark', height=300)
        st.plotly_chart(fig_pie, use_container_width=True)

def zk_proofs_page():
    """Página de provas ZK."""
    st.title("🔐 Provas Zero-Knowledge")

    st.markdown("### 📊 Estatísticas")
    col1, col2, col3 = st.columns(3)
    col1.metric("Provas Geradas", "1,247")
    col2.metric("Verificadas On-Chain", "1,189")
    col3.metric("Taxa de Sucesso", "95.3%")

    st.markdown("### 🔬 Últimas Provas")
    proofs = [
        {"ID": "zk-001", "Circuito": "governance_consensus", "Curva": "BN254", "Status": "✅ On-Chain"},
        {"ID": "zk-002", "Circuito": "policy_evaluation", "Curva": "BLS12-381", "Status": "✅ Off-Chain"},
        {"ID": "zk-003", "Circuito": "governance_consensus", "Curva": "BN254", "Status": "⏳ Pendente"},
    ]
    st.dataframe(pd.DataFrame(proofs), use_container_width=True)

    # Verificação manual
    st.markdown("### 🛠️ Verificar Prova")
    with st.form("verify_form"):
        proof_id = st.text_input("ID da Prova")
        submitted = st.form_submit_button("Verificar")
        if submitted:
            st.success(f"✅ Prova {proof_id} verificada com sucesso (simulado)")

def federation_page():
    """Página de federação e trust scores."""
    st.title("🌐 Governança Federada")

    # Trust scores
    st.markdown("### 📊 Trust Scores dos Peers")
    trust_data = {
        "Peer": ["Catedral-PR", "Catedral-SP", "Catedral-RJ", "Catedral-DF"],
        "Trust Score": [0.92, 0.78, 0.63, 0.45],
        "Status": ["🟢 Ativo", "🟢 Ativo", "🟡 Degradado", "🔴 Inativo"]
    }
    df_trust = pd.DataFrame(trust_data)
    st.dataframe(df_trust, use_container_width=True)

    # Gráfico de trust
    fig_trust = px.bar(df_trust, x="Peer", y="Trust Score", color="Status",
                       color_discrete_map={"🟢 Ativo": "green", "🟡 Degradado": "orange", "🔴 Inativo": "red"})
    st.plotly_chart(fig_trust, use_container_width=True)

    # Sincronização
    if st.button("🔄 Sincronizar Agora"):
        with st.spinner("Sincronizando..."):
            time.sleep(2)
            st.success("✅ Sincronização concluída")

def merkle_tree_page():
    """Página da Merkle Tree federada."""
    st.title("🌳 Merkle Tree Federada")

    st.markdown("### 📜 Estado Atual")
    st.code("""
    Raiz Local: 0x7f3a4b2c1d...e9f0
    Raiz Federada: 0xa1b2c3d4...f5e6
    Blocos: 256
    Última Sincronização: 2026-08-29 14:30:00
    """)

    # Visualização da árvore (simplificada)
    st.markdown("### 🌲 Visualização da Árvore")
    st.markdown("""
    ```
          ┌───────────────┐
          │  Root: 0x7f3a.. │
          └───────┬───────┘
              ┌───┴───┐
          ┌───┴───┐ ┌───┴───┐
          │ 0x1a.. │ │ 0x2b.. │
          └───┬───┘ └───┬───┘
          ┌───┴───┐ ┌───┴───┐
          │ Leaf1 │ │ Leaf2 │ ...
          └───────┘ └───────┘
    ```
    """)

def policy_eval_page():
    """Página de avaliação de políticas com ZK."""
    st.title("📜 Zero-Knowledge Policy Evaluation")

    st.markdown("### 📋 Políticas Ativas")
    policies = [
        "policy_approval(PR_ID) :– coherence(PR_ID, Phi), Phi >= 0.85, not(critical_violation(PR_ID)).",
        "policy_requires_manual_review(PR_ID) :– contains_pii(PR_ID).",
        "policy_autofix_allowed(PR_ID) :– coherence(PR_ID, Phi), Phi >= 0.7."
    ]
    for p in policies:
        st.code(p)

    st.markdown("### 🧪 Avaliar Política")
    with st.form("policy_form"):
        policy = st.selectbox("Política", ["policy_approval", "policy_requires_manual_review", "policy_autofix_allowed"])
        pr_id = st.text_input("PR ID", "PR-123")
        submitted = st.form_submit_button("Avaliar com ZK")

        if submitted:
            with st.spinner("Gerando prova..."):
                time.sleep(1)
                st.success("✅ Política satisfeita")
                st.json({
                    "policy": policy,
                    "pr_id": pr_id,
                    "result": "satisfied",
                    "zk_proof": {
                        "proof_hash": "0x" + hashlib.sha256(f"{policy}:{pr_id}".encode()).hexdigest()[:8],
                        "circuit": "policy_evaluation",
                        "curve": "BN254"
                    }
                })

# ============================================================================
# MAIN
# ============================================================================

def main():
    st.set_page_config(page_title="Catedral OS Governance", layout="wide")

    # Autenticação
    # if not auth_ui():
    #     st.warning("🔑 Por favor, autentique-se com seu DID")
    #     st.stop()

    # Navegação
    pages = {
        "📊 Visão Geral": overview_page,
        "🔐 ZK-Proofs": zk_proofs_page,
        "🌐 Federação": federation_page,
        "🌳 Merkle Tree": merkle_tree_page,
        "📜 Policy Eval": policy_eval_page
    }

    selection = st.sidebar.radio("Navegação", list(pages.keys()))
    pages[selection]()

if __name__ == "__main__":
    main()