//! Saídas DMX (A1, E4): cada interface tem a sua thread, que pega o quadro mais recente do universo
//! dela no próprio ritmo, com prazo absoluto, e conta quadros, atraso e erros para o Monitoramento.
//! A interface só fica aberta enquanto tem universo patchado (fora disso, outro programa pode usá-la),
//! e uma interface que some (desplugada) é reaberta a cada segundo até voltar.

pub mod ftdi;

use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

/// Uma interface DMX aberta.
pub trait Device: Send {
    /// Envia um quadro de 512 canais.
    fn send(&mut self, frame: &[u8; 512]) -> io::Result<()>;
    /// Quadros por segundo desta interface.
    fn rate_hz(&self) -> u32;
}

/// Interface virtual: aceita os quadros sem mandar nada, para desenvolver e testar sem hardware (E4).
pub struct Dummy;

impl Device for Dummy {
    fn send(&mut self, _: &[u8; 512]) -> io::Result<()> {
        Ok(())
    }

    fn rate_hz(&self) -> u32 {
        44
    }
}

/// Contadores da saída, lidos pelo servidor a cada segundo.
#[derive(Default)]
pub struct Stats {
    pub frames: AtomicU64,
    pub errors: AtomicU32,
    /// Maior desvio do intervalo entre dois quadros desde a última leitura, em µs.
    pub jitter_us: AtomicU32,
    /// A interface está aberta e mandando.
    pub open: AtomicBool,
    /// A interface respondeu como uma Enttec Pro (detecção automática).
    pub pro: AtomicBool,
}

/// Thread de uma saída. Parar o `Output` (drop) fecha a interface e encerra a thread no próximo quadro.
pub struct Output {
    pub stats: Arc<Stats>,
    stop: Arc<AtomicBool>,
}

impl Drop for Output {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

/// Inicia a saída. `open` abre a interface (pode anotar nos contadores o que descobriu dela) e
/// `source` dá o quadro do universo patchado nela (`None` = nenhum: a interface fica fechada).
pub fn spawn(
    name: &str,
    mut open: impl FnMut(&Stats) -> io::Result<Box<dyn Device>> + Send + 'static,
    mut source: impl FnMut() -> Option<[u8; 512]> + Send + 'static,
) -> Output {
    let stats = Arc::new(Stats::default());
    let stop = Arc::new(AtomicBool::new(false));
    let (counters, stopped) = (stats.clone(), stop.clone());
    thread::Builder::new()
        .name(format!("saida-{name}"))
        .spawn(move || {
            realtime_priority();
            let mut device: Option<Box<dyn Device>> = None;
            let mut retry = Instant::now();
            let mut deadline = Instant::now();
            let mut last: Option<Instant> = None;
            while !stopped.load(Ordering::Relaxed) {
                let frame = source();
                if frame.is_none() {
                    device = None;
                }
                if frame.is_some() && device.is_none() && Instant::now() >= retry {
                    device = open(&counters).ok();
                    retry = Instant::now() + Duration::from_secs(1);
                }
                if let (Some(d), Some(frame)) = (device.as_mut(), frame) {
                    let now = Instant::now();
                    if let Some(last) = last {
                        let period = Duration::from_secs(1) / d.rate_hz();
                        let jitter = now.duration_since(last).abs_diff(period).as_micros();
                        counters
                            .jitter_us
                            .fetch_max(jitter as u32, Ordering::Relaxed);
                    }
                    last = Some(now);
                    if d.send(&frame).is_ok() {
                        counters.frames.fetch_add(1, Ordering::Relaxed);
                    } else {
                        // Desplugada ou travada: fecha e tenta reabrir daqui a 1 s
                        counters.errors.fetch_add(1, Ordering::Relaxed);
                        device = None;
                    }
                }
                if device.is_none() {
                    last = None;
                }
                counters.open.store(device.is_some(), Ordering::Relaxed);
                // Prazo absoluto: um quadro atrasado é compensado no seguinte, sem deriva. Sem
                // interface aberta, o ritmo é o mesmo, só para olhar o patch.
                let period = Duration::from_secs(1) / device.as_ref().map_or(44, |d| d.rate_hz());
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

/// As saídas dormem quase o tempo todo; com a prioridade mais alta, acordam na hora mesmo com o
/// computador ocupado.
fn realtime_priority() {
    #[cfg(windows)]
    // SAFETY: muda só a prioridade da própria thread.
    unsafe {
        use windows_sys::Win32::System::Threading::{
            GetCurrentThread, SetThreadPriority, THREAD_PRIORITY_TIME_CRITICAL,
        };
        SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_TIME_CRITICAL);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dummy_sends_at_its_rate_and_only_when_patched() {
        let patched = Arc::new(AtomicBool::new(true));
        let flag = patched.clone();
        let output = spawn(
            "teste",
            |_| Ok(Box::new(Dummy)),
            move || flag.load(Ordering::Relaxed).then_some([0; 512]),
        );
        thread::sleep(Duration::from_millis(500));
        let sent = output.stats.frames.load(Ordering::Relaxed);
        // 44 Hz por 0,5 s: ~22 quadros (folga para o agendador do sistema)
        assert!((17..=27).contains(&sent), "{sent} quadros");
        assert!(output.stats.open.load(Ordering::Relaxed));
        patched.store(false, Ordering::Relaxed);
        thread::sleep(Duration::from_millis(60));
        let before = output.stats.frames.load(Ordering::Relaxed);
        thread::sleep(Duration::from_millis(200));
        assert_eq!(output.stats.frames.load(Ordering::Relaxed), before);
        assert!(!output.stats.open.load(Ordering::Relaxed));
    }

    /// Interface que falha num quadro e volta depois.
    struct Flaky(Arc<AtomicU32>);

    impl Device for Flaky {
        fn send(&mut self, _: &[u8; 512]) -> io::Result<()> {
            match self.0.fetch_add(1, Ordering::Relaxed) {
                3 => Err(io::Error::other("desplugada")),
                _ => Ok(()),
            }
        }

        fn rate_hz(&self) -> u32 {
            100
        }
    }

    #[test]
    fn a_failing_interface_is_reopened() {
        let sends = Arc::new(AtomicU32::new(0));
        let opens = Arc::new(AtomicU32::new(0));
        let (s, o) = (sends.clone(), opens.clone());
        let output = spawn(
            "falha",
            move |_| {
                o.fetch_add(1, Ordering::Relaxed);
                Ok(Box::new(Flaky(s.clone())))
            },
            || Some([0; 512]),
        );
        thread::sleep(Duration::from_millis(1500));
        assert_eq!(output.stats.errors.load(Ordering::Relaxed), 1);
        // Reaberta 1 s depois da falha, e voltou a mandar
        assert_eq!(opens.load(Ordering::Relaxed), 2);
        assert!(output.stats.open.load(Ordering::Relaxed));
        assert!(output.stats.frames.load(Ordering::Relaxed) > 10);
    }
}
