//! Saídas DMX (A1, E4): cada interface tem a sua thread, que pega o quadro mais recente do universo
//! dela no próprio ritmo, com prazo absoluto, e conta quadros, atraso e erros para o Monitoramento.
//! Na rodada 1 da F3 só existe a Dummy; a Enttec Pro e a Open DMX chegam na rodada 2.

use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

/// Uma interface DMX aberta.
pub trait Device: Send + 'static {
    /// Envia um quadro de 512 canais.
    fn send(&mut self, frame: &[u8; 512]) -> io::Result<()>;
}

/// Interface virtual: aceita os quadros sem mandar nada, para desenvolver e testar sem hardware (E4).
pub struct Dummy;

impl Device for Dummy {
    fn send(&mut self, _: &[u8; 512]) -> io::Result<()> {
        Ok(())
    }
}

/// Contadores da saída, lidos pelo servidor a cada segundo.
#[derive(Default)]
pub struct Stats {
    pub frames: AtomicU64,
    pub errors: AtomicU32,
    /// Maior desvio do intervalo entre dois quadros desde a última leitura, em µs.
    pub jitter_us: AtomicU32,
}

/// Thread de uma saída. Parar o `Output` (drop) encerra a thread no próximo quadro.
pub struct Output {
    pub stats: Arc<Stats>,
    stop: Arc<AtomicBool>,
}

impl Drop for Output {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

/// Inicia a saída: `rate_hz` vezes por segundo pede o quadro a `source` e o envia ao aparelho.
/// Sem universo patchado (`None`), não envia nada.
pub fn spawn(
    name: &str,
    mut device: impl Device,
    rate_hz: u32,
    mut source: impl FnMut() -> Option<[u8; 512]> + Send + 'static,
) -> Output {
    let stats = Arc::new(Stats::default());
    let stop = Arc::new(AtomicBool::new(false));
    let (counters, stopped) = (stats.clone(), stop.clone());
    let period = Duration::from_secs(1) / rate_hz;
    thread::Builder::new()
        .name(format!("saida-{name}"))
        .spawn(move || {
            let mut deadline = Instant::now();
            let mut last: Option<Instant> = None;
            while !stopped.load(Ordering::Relaxed) {
                if let Some(frame) = source() {
                    let now = Instant::now();
                    if let Some(last) = last {
                        let jitter = now.duration_since(last).abs_diff(period).as_micros();
                        counters
                            .jitter_us
                            .fetch_max(jitter as u32, Ordering::Relaxed);
                    }
                    last = Some(now);
                    if device.send(&frame).is_ok() {
                        counters.frames.fetch_add(1, Ordering::Relaxed);
                    } else {
                        counters.errors.fetch_add(1, Ordering::Relaxed);
                    }
                } else {
                    last = None;
                }
                // Prazo absoluto: um quadro atrasado é compensado no seguinte, sem deriva
                deadline += period;
                let now = Instant::now();
                if deadline + period <= now {
                    deadline = now;
                }
                thread::sleep(deadline.saturating_duration_since(now));
            }
        })
        .expect("não foi possível iniciar a thread da saída");
    Output { stats, stop }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dummy_sends_at_the_rate_and_only_when_patched() {
        let patched = Arc::new(AtomicBool::new(true));
        let flag = patched.clone();
        let output = spawn("teste", Dummy, 100, move || {
            flag.load(Ordering::Relaxed).then_some([0; 512])
        });
        thread::sleep(Duration::from_millis(500));
        let sent = output.stats.frames.load(Ordering::Relaxed);
        // 100 Hz por 0,5 s: ~50 quadros (folga para o agendador do sistema)
        assert!((40..=60).contains(&sent), "{sent} quadros");
        patched.store(false, Ordering::Relaxed);
        thread::sleep(Duration::from_millis(50));
        let before = output.stats.frames.load(Ordering::Relaxed);
        thread::sleep(Duration::from_millis(200));
        assert_eq!(output.stats.frames.load(Ordering::Relaxed), before);
        assert_eq!(output.stats.errors.load(Ordering::Relaxed), 0);
    }
}
