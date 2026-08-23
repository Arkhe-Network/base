// src/kani_proofs.rs
//! Harnesses Kani para verificação formal do DEP v2.5
//! Compilados apenas quando --cfg kani está ativo (cargo kani)

use crate::{
    types::*,
    acquisition::Acquisition,
    storage::Storage,
    labeling::Labeling,
    consensus::Consensus,
    audit::AuditTrail,
    rem::RemWindow,
    byzantine::ByzantineDetector,
};

// ─── Proofs de Aquisição ───────────────────────────────────────────────

#[kani::proof]
fn acquire_never_panics() {
    let config = Config::default();
    let mut acquisition = Acquisition::new(&config);

    // Kani só suporta primitivos. Usamos u8 para sensor (0-3) e arrays fixos.
    let ts: u8 = kani::any();
    let sensor_code: u8 = kani::any();
    let sample0: u8 = kani::any();
    let sample1: u8 = kani::any();
    let now: u8 = kani::any();

    kani::assume(ts < 50);
    kani::assume(now < 50);
    kani::assume(sensor_code < 3); // 0=EEG, 1=EOG, 2=EMG (excluímos STIM para evitar REM)
    kani::assume(ts >= now.saturating_sub(2));
    kani::assume(ts <= now + 2);

    let sensor = match sensor_code {
        0 => SensorType::EEG,
        1 => SensorType::EOG,
        2 => SensorType::EMG,
        _ => SensorType::EEG, // unreachable por assume
    };

    let sample = vec![sample0 as Sample, sample1 as Sample];
    let _ = acquisition.acquire(ts as Timestamp, sensor, &sample, now as Timestamp);
}

#[kani::proof]
fn update_rem_window_never_panics() {
    let config = Config::default();
    let mut acquisition = Acquisition::new(&config);
    let start: u8 = kani::any();
    let end: u8 = kani::any();
    let now: u8 = kani::any();

    kani::assume(start < 20);
    kani::assume(end < 20);
    kani::assume(now < 20);
    kani::assume(end > start);
    kani::assume((end - start) >= 1);
    kani::assume((end - start) <= 3);

    let _ = acquisition.update_rem_window(start as Timestamp, end as Timestamp, now as Timestamp);
}

#[kani::proof]
fn hardware_failure_recovery_never_panics() {
    let config = Config::default();
    let mut acquisition = Acquisition::new(&config);

    acquisition.hardware_failure();
    assert!(!acquisition.is_hardware_active());

    acquisition.hardware_recovery();
    assert!(acquisition.is_hardware_active());
}

// ─── Proofs de Storage ─────────────────────────────────────────────────

#[kani::proof]
fn close_epoch_never_panics() {
    let config = Config::default();
    let mut storage = Storage::new();
    let start_ts: u8 = kani::any();

    let packet1 = DataPacket::new(0, SensorType::EEG, vec![0, 1], 0);
    let packet2 = DataPacket::new(1, SensorType::EOG, vec![1, 0], 1);
    let packets = vec![packet1, packet2];

    kani::assume(start_ts < 50);
    let _ = storage.close_epoch(start_ts as Timestamp, packets, &config);
}

#[kani::proof]
fn get_epoch_never_panics() {
    let storage = Storage::new();
    let id: u8 = kani::any();
    kani::assume(id < 10);
    let _ = storage.get_epoch(id as EpochId);
}

// ─── Proofs de Labeling ────────────────────────────────────────────────

#[kani::proof]
fn assign_label_never_panics() {
    let config = Config::default();
    let mut labeling = Labeling::new();
    let rater_id: u8 = kani::any();
    let epoch_id: u8 = kani::any();
    let verdict_code: u8 = kani::any();
    let timestamp: u8 = kani::any();

    kani::assume(epoch_id < 10);
    kani::assume(timestamp < 50);
    kani::assume(verdict_code < 3);
    kani::assume(rater_id < 5); // R0-R4, assumindo que E1,E2 são > 4

    let rater = format!("R{}", rater_id);
    let verdict = match verdict_code {
        0 => Verdict::Lucid,
        1 => Verdict::NonLucid,
        _ => Verdict::Inconclusive,
    };

    let _ = labeling.assign_label(rater, epoch_id as EpochId, verdict, timestamp as Timestamp, &config);
}

#[kani::proof]
fn process_label_never_panics() {
    let _config = Config::default();
    let mut labeling = Labeling::new();
    let rater = "R1".to_string();
    let epoch_id = 0;
    let verdict = Verdict::Lucid;
    let signature = sign(&rater, epoch_id, verdict);
    let label = RaterLabel::new(rater, epoch_id, verdict, 0, signature);

    labeling.labels.insert(label.clone());
    let _ = labeling.process_label(&label);
}

// ─── Proofs de Consenso ────────────────────────────────────────────────

#[kani::proof]
fn reach_threshold_never_panics() {
    let config = Config::default();
    let mut consensus = Consensus::new(&config);
    let mut labeling = Labeling::new();
    let epoch_id: u8 = kani::any();

    kani::assume(epoch_id < 10);

    // Adicionar labels suficientes (threshold = 3)
    for i in 0..3u8 {
        let rater = format!("R{}", i);
        let verdict = Verdict::Lucid;
        let signature = sign(&rater, epoch_id as EpochId, verdict);
        let label = RaterLabel::new(rater, epoch_id as EpochId, verdict, 0, signature);
        let _ = labeling.process_label(&label);
    }

    let _ = consensus.reach_threshold(epoch_id as EpochId, &labeling);
}

#[kani::proof]
fn get_result_never_panics() {
    let config = Config::default();
    let consensus = Consensus::new(&config);
    let id: u8 = kani::any();
    kani::assume(id < 10);
    let _ = consensus.get_result(id as EpochId);
}

// ─── Proofs de Audit ───────────────────────────────────────────────────

#[kani::proof]
fn append_and_verify_chain_never_panics() {
    let mut audit = AuditTrail::new();
    let hash1 = hash_data(b"hash1");
    let hash2 = hash_data(b"hash2");

    let _ = audit.append(hash1, 0);
    let _ = audit.append(hash2, 1);
    let _ = audit.verify_chain();
}

#[kani::proof]
fn root_hash_never_panics() {
    let audit = AuditTrail::new();
    let _ = audit.root_hash();
}

// ─── Proofs de REM ─────────────────────────────────────────────────────

#[kani::proof]
fn rem_window_update_never_panics() {
    let _config = Config::default();
    let mut rem = RemWindow::new();
    let start: u8 = kani::any();
    let end: u8 = kani::any();
    let now: u8 = kani::any();

    kani::assume(start < 20);
    kani::assume(end < 20);
    kani::assume(now < 20);
    kani::assume(end > start);
    kani::assume((end - start) >= 1);
    kani::assume((end - start) <= 3);

    let _ = rem.update(start as Timestamp, end as Timestamp, now as Timestamp, &_config);
}

#[kani::proof]
fn rem_is_active_never_panics() {
    let rem = RemWindow::new();
    let ts: u8 = kani::any();
    kani::assume(ts < 50);
    let _ = rem.is_active(ts as Timestamp);
}

// ─── Proofs de Byzantine ───────────────────────────────────────────────

#[kani::proof]
fn detect_forgery_never_panics() {
    let config = Config::default();
    let mut detector = ByzantineDetector::new(&config);
    let labeling = Labeling::new();
    let _ = detector.detect_forgery(&labeling);
}

#[kani::proof]
fn is_trustworthy_never_panics() {
    let config = Config::default();
    let detector = ByzantineDetector::new(&config);
    let rater_id: u8 = kani::any();
    let rater = format!("R{}", rater_id);
    let _ = detector.is_trustworthy(&rater);
}