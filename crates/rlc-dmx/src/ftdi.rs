//! Interfaces USB com chip FTDI, pelo driver D2XX da FTDI (E4). A biblioteca vem do driver instalado
//! no sistema (`ftd2xx.dll`, que o Windows instala com a interface), carregada ao usar: sem ela, não
//! há interfaces USB e a Dummy continua funcionando.
//!
//! Dois protocolos, como no plugin `dmxusb` do QLC+ (referência):
//! - **Enttec DMX USB Pro** e compatíveis: um microcontrolador na interface gera o DMX; o app manda
//!   pacotes `0x7E | rótulo | tamanho | dados | 0xE7` (rótulo 6 = quadro DMX).
//! - **Open DMX** (só o chip FTDI): o app gera o sinal, com break e MAB antes de cada quadro.

use std::ffi::{CStr, c_void};
use std::io;
use std::sync::OnceLock;
use std::thread;
use std::time::{Duration, Instant};

use libloading::Library;

use crate::{Device, Stats};

/// Interface FTDI ligada ao computador.
#[derive(Clone, Debug, PartialEq)]
pub struct FtdiInfo {
    pub serial: String,
    /// Nome gravado na interface (ex.: "DMX USB PRO", "INTERFACE DMX - NUMBSTAR").
    pub description: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Protocol {
    EnttecPro,
    OpenDmx,
}

/// Protocolo pelo nome, como no QLC+: "DMX USB PRO" (e a Mk2) é a Pro; o resto, Open DMX.
pub fn protocol_by_name(description: &str) -> Protocol {
    let name = description.to_uppercase();
    match name.contains("DMX USB PRO") || name.contains("PRO MK2") {
        true => Protocol::EnttecPro,
        false => Protocol::OpenDmx,
    }
}

/// Interfaces FTDI ligadas agora (nenhuma, se o driver não estiver instalado).
pub fn devices() -> Vec<FtdiInfo> {
    let Some(api) = api() else { return vec![] };
    let mut count = 0;
    // SAFETY: chamadas da D2XX com ponteiros para variáveis e buffers locais, do tamanho que ela pede.
    unsafe {
        if (api.create_device_info_list)(&mut count) != OK {
            return vec![];
        }
        (0..count)
            .filter_map(|i| {
                let (mut flags, mut kind, mut id, mut location) = (0, 0, 0, 0);
                let mut serial = [0u8; 16];
                let mut description = [0u8; 64];
                let mut handle = std::ptr::null_mut();
                let status = (api.get_device_info_detail)(
                    i,
                    &mut flags,
                    &mut kind,
                    &mut id,
                    &mut location,
                    serial.as_mut_ptr().cast(),
                    description.as_mut_ptr().cast(),
                    &mut handle,
                );
                let text = |b: &[u8]| {
                    CStr::from_bytes_until_nul(b).map(|s| s.to_string_lossy().trim().to_string())
                };
                match (status, text(&serial), text(&description)) {
                    (OK, Ok(serial), Ok(description)) if !serial.is_empty() => Some(FtdiInfo {
                        serial,
                        description,
                    }),
                    _ => None,
                }
            })
            .collect()
    }
}

/// Abre a interface pelo número de série e prepara a linha DMX (250 000 baud, 8N2, sem controle de
/// fluxo). Em `auto` com a Open DMX, primeiro pergunta o número de série no protocolo da Pro: uma
/// interface que responde é uma Pro compatível, mesmo sem o nome dela, e vale para esta sessão.
pub fn open(
    serial: &str,
    protocol: Protocol,
    auto: bool,
    stats: &Stats,
) -> io::Result<Box<dyn Device>> {
    let port = Port::open(serial)?;
    let pro = protocol == Protocol::EnttecPro
        || stats.pro.load(std::sync::atomic::Ordering::Relaxed)
        || (auto && port.answers_like_pro());
    stats.pro.store(pro, std::sync::atomic::Ordering::Relaxed);
    Ok(match pro {
        true => Box::new(EnttecPro(port)),
        false => Box::new(OpenDmx(port)),
    })
}

struct EnttecPro(Port);

impl Device for EnttecPro {
    fn send(&mut self, frame: &[u8; 512]) -> io::Result<()> {
        self.0.write(&pro_packet(frame))
    }

    /// Como o QLC+: 44 quadros por segundo; o DMX no fio é gerado pela própria interface.
    fn rate_hz(&self) -> u32 {
        44
    }
}

struct OpenDmx(Port);

impl Device for OpenDmx {
    /// Break de 110 µs e MAB de 16 µs, com espera ativa (como o QLC+), e o quadro com o código de
    /// início. Cada chamada ao USB leva perto de 1 ms, então o break e o MAB reais ficam nessa ordem,
    /// ainda dentro do padrão (break de 88 µs para cima).
    fn send(&mut self, frame: &[u8; 512]) -> io::Result<()> {
        self.0.call(|api, h| unsafe { (api.set_break_on)(h) })?;
        spin(Duration::from_micros(110));
        self.0.call(|api, h| unsafe { (api.set_break_off)(h) })?;
        spin(Duration::from_micros(16));
        let mut data = [0u8; 513];
        data[1..].copy_from_slice(frame);
        self.0.write(&data)
    }

    /// 30 quadros por segundo, como o QLC+: os 513 bytes levam ~23 ms no fio e precisam terminar
    /// antes do próximo break.
    fn rate_hz(&self) -> u32 {
        30
    }
}

/// Pacote da Pro com um quadro DMX: rótulo 6, tamanho 513 (código de início + 512 canais).
pub fn pro_packet(frame: &[u8; 512]) -> [u8; 518] {
    let mut packet = [0u8; 518];
    packet[..5].copy_from_slice(&[0x7E, 6, 0x01, 0x02, 0x00]);
    packet[5..517].copy_from_slice(frame);
    packet[517] = 0xE7;
    packet
}

/// Resposta da Pro ao pedido do número de série (rótulo 10).
fn is_pro_reply(reply: &[u8]) -> bool {
    reply.len() >= 5 && reply[0] == 0x7E && reply[1] == 10 && reply.contains(&0xE7)
}

fn spin(duration: Duration) {
    let end = Instant::now() + duration;
    while Instant::now() < end {
        std::hint::spin_loop();
    }
}

/// Interface aberta; fechada ao sair de cena.
struct Port(Handle);

#[derive(Clone, Copy)]
struct Handle(*mut c_void);

// SAFETY: a D2XX aceita o handle em qualquer thread; cada interface é usada por uma thread só.
unsafe impl Send for Handle {}

impl Port {
    fn open(serial: &str) -> io::Result<Self> {
        let api = api().ok_or_else(|| io::Error::other("driver D2XX da FTDI não encontrado"))?;
        let serial = std::ffi::CString::new(serial).map_err(io::Error::other)?;
        let mut handle = std::ptr::null_mut();
        // SAFETY: número de série como C string e ponteiro para o handle de saída.
        let status = unsafe {
            (api.open_ex)(
                serial.as_ptr().cast_mut().cast(),
                OPEN_BY_SERIAL_NUMBER,
                &mut handle,
            )
        };
        if status != OK {
            return Err(io::Error::other(format!("não abriu (D2XX {status})")));
        }
        let port = Port(Handle(handle));
        port.call(|api, h| unsafe { (api.reset_device)(h) })?;
        port.call(|api, h| unsafe { (api.set_baud_rate)(h, 250_000) })?;
        port.call(|api, h| unsafe { (api.set_data_characteristics)(h, 8, STOP_BITS_2, 0) })?;
        port.call(|api, h| unsafe { (api.set_flow_control)(h, 0, 0, 0) })?;
        // Uma interface travada devolve erro em vez de prender a thread
        port.call(|api, h| unsafe { (api.set_timeouts)(h, 100, 100) })?;
        port.call(|api, h| unsafe { (api.purge)(h, PURGE_RX | PURGE_TX) })?;
        port.call(|api, h| unsafe { (api.clr_rts)(h) })?;
        Ok(port)
    }

    /// Chama uma função da D2XX com o handle e transforma o status em erro. As chamadas são `unsafe`
    /// (FFI); o handle vale enquanto o `Port` existe.
    fn call(&self, f: impl FnOnce(&Api, *mut c_void) -> u32) -> io::Result<()> {
        let api = api().ok_or_else(|| io::Error::other("driver D2XX ausente"))?;
        match f(api, self.0.0) {
            OK => Ok(()),
            status => Err(io::Error::other(format!("D2XX {status}"))),
        }
    }

    fn write(&self, data: &[u8]) -> io::Result<()> {
        let mut written = 0;
        // SAFETY: buffer válido durante a chamada, tamanho exato.
        self.call(|api, h| unsafe {
            (api.write)(
                h,
                data.as_ptr().cast_mut().cast(),
                data.len() as u32,
                &mut written,
            )
        })?;
        match written as usize == data.len() {
            true => Ok(()),
            false => Err(io::Error::other("escrita incompleta")),
        }
    }

    /// Pergunta o número de série no protocolo da Pro e espera a resposta por 100 ms. Numa Open DMX,
    /// os 5 bytes vão para o fio sem break, e as fixtures os ignoram.
    fn answers_like_pro(&self) -> bool {
        if self.write(&[0x7E, 10, 0, 0, 0xE7]).is_err() {
            return false;
        }
        thread::sleep(Duration::from_millis(20));
        let mut reply = [0u8; 9];
        let mut read = 0;
        // SAFETY: buffer local do tamanho pedido.
        let ok = self.call(|api, h| unsafe {
            (api.read)(h, reply.as_mut_ptr().cast(), reply.len() as u32, &mut read)
        });
        let _ = self.call(|api, h| unsafe { (api.purge)(h, PURGE_RX | PURGE_TX) });
        ok.is_ok() && is_pro_reply(&reply[..read as usize])
    }
}

impl Drop for Port {
    fn drop(&mut self) {
        let _ = self.call(|api, h| unsafe { (api.close)(h) });
    }
}

// ---------- Funções da D2XX ----------

const OK: u32 = 0;
const OPEN_BY_SERIAL_NUMBER: u32 = 1;
const STOP_BITS_2: u8 = 2;
const PURGE_RX: u32 = 1;
const PURGE_TX: u32 = 2;

type H = *mut c_void;

struct Api {
    _library: Library,
    create_device_info_list: unsafe extern "system" fn(*mut u32) -> u32,
    get_device_info_detail: unsafe extern "system" fn(
        u32,
        *mut u32,
        *mut u32,
        *mut u32,
        *mut u32,
        *mut c_void,
        *mut c_void,
        *mut H,
    ) -> u32,
    open_ex: unsafe extern "system" fn(*mut c_void, u32, *mut H) -> u32,
    close: unsafe extern "system" fn(H) -> u32,
    reset_device: unsafe extern "system" fn(H) -> u32,
    set_baud_rate: unsafe extern "system" fn(H, u32) -> u32,
    set_data_characteristics: unsafe extern "system" fn(H, u8, u8, u8) -> u32,
    set_flow_control: unsafe extern "system" fn(H, u16, u8, u8) -> u32,
    set_timeouts: unsafe extern "system" fn(H, u32, u32) -> u32,
    purge: unsafe extern "system" fn(H, u32) -> u32,
    clr_rts: unsafe extern "system" fn(H) -> u32,
    set_break_on: unsafe extern "system" fn(H) -> u32,
    set_break_off: unsafe extern "system" fn(H) -> u32,
    write: unsafe extern "system" fn(H, *mut c_void, u32, *mut u32) -> u32,
    read: unsafe extern "system" fn(H, *mut c_void, u32, *mut u32) -> u32,
}

/// A D2XX, carregada uma vez. `None` quando o driver não está instalado.
fn api() -> Option<&'static Api> {
    static API: OnceLock<Option<Api>> = OnceLock::new();
    API.get_or_init(|| {
        let name = if cfg!(windows) {
            "ftd2xx.dll"
        } else if cfg!(target_os = "macos") {
            "libftd2xx.dylib"
        } else {
            "libftd2xx.so"
        };
        // SAFETY: a biblioteca do driver da FTDI, com as assinaturas da documentação da D2XX.
        let loaded = unsafe {
            let library = Library::new(name).ok()?;
            macro_rules! f {
                ($name:literal) => {
                    *library.get($name).ok()?
                };
            }
            Some(Api {
                create_device_info_list: f!(b"FT_CreateDeviceInfoList\0"),
                get_device_info_detail: f!(b"FT_GetDeviceInfoDetail\0"),
                open_ex: f!(b"FT_OpenEx\0"),
                close: f!(b"FT_Close\0"),
                reset_device: f!(b"FT_ResetDevice\0"),
                set_baud_rate: f!(b"FT_SetBaudRate\0"),
                set_data_characteristics: f!(b"FT_SetDataCharacteristics\0"),
                set_flow_control: f!(b"FT_SetFlowControl\0"),
                set_timeouts: f!(b"FT_SetTimeouts\0"),
                purge: f!(b"FT_Purge\0"),
                clr_rts: f!(b"FT_ClrRts\0"),
                set_break_on: f!(b"FT_SetBreakOn\0"),
                set_break_off: f!(b"FT_SetBreakOff\0"),
                write: f!(b"FT_Write\0"),
                read: f!(b"FT_Read\0"),
                _library: library,
            })
        };
        if loaded.is_none() {
            eprintln!("driver D2XX da FTDI ({name}) não encontrado: sem interfaces USB");
        }
        loaded
    })
    .as_ref()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pro_packet_carries_start_code_and_512_channels() {
        let mut frame = [0u8; 512];
        frame[0] = 255;
        frame[511] = 7;
        let p = pro_packet(&frame);
        // Rótulo 6 e tamanho 513 (0x0201), em little endian
        assert_eq!(p[..5], [0x7E, 6, 0x01, 0x02, 0x00]);
        assert_eq!((p[5], p[516], p[517]), (255, 7, 0xE7));
    }

    #[test]
    fn protocol_by_name_follows_qlc() {
        assert_eq!(protocol_by_name("DMX USB PRO"), Protocol::EnttecPro);
        assert_eq!(protocol_by_name("dmx usb pro mk2"), Protocol::EnttecPro);
        assert_eq!(
            protocol_by_name("INTERFACE DMX - NUMBSTAR"),
            Protocol::OpenDmx
        );
        assert_eq!(protocol_by_name("FT232R USB UART"), Protocol::OpenDmx);
    }

    #[test]
    fn pro_reply_is_recognized() {
        assert!(is_pro_reply(&[
            0x7E, 10, 4, 0, 0x12, 0x34, 0x56, 0x78, 0xE7
        ]));
        assert!(!is_pro_reply(&[]));
        assert!(!is_pro_reply(&[0x7E, 5, 0, 0, 0xE7]));
    }
}
