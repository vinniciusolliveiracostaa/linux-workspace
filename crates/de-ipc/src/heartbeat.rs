//! Heartbeat — Detecção de componentes mortos
//!
//! # Como funciona
//! 1. IpcClient spawna task que envia Message::Heartbeat a cada INTERVAL
//! 2. IpcBus monitora timestamps — se um componente não envia heartbeat
//!    por mais de TIMEOUT, ele é considerado morto
//!
//! # POR QUE heartbeat e não TCP keepalive?
//! - Unix Domain Sockets não suportam TCP keepalive
//! - Heartbeat é application-level: detecta componente TRAVADO (não só desconectado)
//! - Componente pode estar vivo na rede mas em deadlock — heartbeat detecta isso
//!
//! # POR QUE constantes aqui e não em cada módulo?
//! - Single Source of Truth: INTERVAL e TIMEOUT devem ser consistentes
//! - Se mudar um, deve mudar o outro (TIMEOUT = 3 * INTERVAL é a regra)
//! - Centralizar evita bugs onde client envia a cada 10s mas server espera 5s

use std::time::Duration;

/// Intervalo entre heartbeats enviados pelo client
///
/// POR QUE 5s?
/// - Frequente o suficiente para detectar crashes rápido (~15s no pior caso)
/// - Raro o suficiente para não gerar overhead de I/O perceptível
/// - 3 heartbeats perdidos = timeout (5 * 3 = 15s)
pub const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// Timeout para considerar um componente morto
///
/// POR QUE 15s (3x o intervalo)?
/// - 1 heartbeat perdido = rede lenta ou GC momentâneo
/// - 2 heartbeats perdidos = algo pode estar errado
/// - 3 heartbeats perdidos = componente está morto ou travado
/// - Regra dos "3 strikes" é padrão em sistemas distribuídos
pub const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(15);
