use thiserror::Error;

#[derive(Error, Debug)]
pub enum TimechainError {
    #[error("Erro de serialização: {0}")]
    SerializationError(#[from] bincode::Error),
    #[error("Erro de rede IO: {0}")]
    NetworkError(#[from] std::io::Error),
    #[error("Condição CFL violada: dt={dt} > max={max}")]
    CflViolation { dt: f64, max: f64 },
    #[error("Sombra inativa ou vazia")]
    EmptyShadow,
    #[error("Assinatura Pós-Quântica inválida ou corrompida")]
    QuantumSignatureInvalid,
    #[error("Falha no estabelecimento de chave quântica (KEM)")]
    QuantumKemFailure,
    #[error("Erro PQC Geral: {0}")]
    PqcError(String),
}
