//! Flash-backed persistence for pointing-device runtime settings.
//!
//! Two 4-KiB slots sit immediately below rmk's storage region (which starts
//! at 0x`A0000`). Each slot carries a 16-byte blob; the rest of the sector is
//! left erased.

use embassy_nrf::nvmc::FLASH_SIZE;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::Mutex;
use embedded_storage_async::nor_flash::{ErrorType, NorFlash, ReadNorFlash};
use nrf_mpsl::{Flash, FlashError};

pub const SLOT_1_ADDR: u32 = 0x9_E000;
pub const SLOT_2_ADDR: u32 = 0x9_F000;
const SLOT_SIZE: u32 = 0x1000;
const BLOB_LEN: usize = 16;
const MAGIC: [u8; 4] = *b"PTS1";
const VERSION: u8 = 1;

pub type SharedFlashMutex = Mutex<ThreadModeRawMutex, Flash<'static>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PointingSettingsSnapshot {
    pub cpi_step: u8,
    pub scroll_step: u8,
    pub cursor_enabled: bool,
    pub scroll_invert_wheel: bool,
    pub scroll_invert_pan: bool,
}

impl PointingSettingsSnapshot {
    fn encode(&self) -> [u8; BLOB_LEN] {
        let mut buf = [0u8; BLOB_LEN];
        buf[0..4].copy_from_slice(&MAGIC);
        buf[4] = VERSION;
        buf[5] = self.cpi_step;
        buf[6] = self.scroll_step;
        let mut flags = 0u8;
        if self.cursor_enabled {
            flags |= 0b0000_0001;
        }
        if self.scroll_invert_wheel {
            flags |= 0b0000_0010;
        }
        if self.scroll_invert_pan {
            flags |= 0b0000_0100;
        }
        buf[7] = flags;
        buf
    }

    fn decode(buf: &[u8; BLOB_LEN]) -> Option<Self> {
        if buf[0..4] != MAGIC || buf[4] != VERSION {
            return None;
        }
        let flags = buf[7];
        Some(Self {
            cpi_step: buf[5],
            scroll_step: buf[6],
            cursor_enabled: flags & 0b0000_0001 != 0,
            scroll_invert_wheel: flags & 0b0000_0010 != 0,
            scroll_invert_pan: flags & 0b0000_0100 != 0,
        })
    }
}

fn slot_addr(slot_index: u8) -> u32 {
    match slot_index {
        1 => SLOT_1_ADDR,
        _ => SLOT_2_ADDR,
    }
}

pub struct SharedFlash {
    mutex: &'static SharedFlashMutex,
}

impl SharedFlash {
    pub const fn new(mutex: &'static SharedFlashMutex) -> Self {
        Self { mutex }
    }
}

impl ErrorType for SharedFlash {
    type Error = FlashError;
}

impl ReadNorFlash for SharedFlash {
    const READ_SIZE: usize = 1;

    async fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        let mut guard = self.mutex.lock().await;
        ReadNorFlash::read(&mut *guard, offset, bytes).await
    }

    fn capacity(&self) -> usize {
        FLASH_SIZE
    }
}

impl NorFlash for SharedFlash {
    const WRITE_SIZE: usize = 4;
    const ERASE_SIZE: usize = 4096;

    async fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        let mut guard = self.mutex.lock().await;
        NorFlash::erase(&mut *guard, from, to).await
    }

    async fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        let mut guard = self.mutex.lock().await;
        NorFlash::write(&mut *guard, offset, bytes).await
    }
}

pub async fn save_slot(
    mutex: &'static SharedFlashMutex,
    slot_index: u8,
    snapshot: PointingSettingsSnapshot,
) -> Result<(), FlashError> {
    let addr = slot_addr(slot_index);
    let blob = snapshot.encode();
    let mut guard = mutex.lock().await;
    NorFlash::erase(&mut *guard, addr, addr + SLOT_SIZE).await?;
    NorFlash::write(&mut *guard, addr, &blob).await
}

pub async fn load_slot(
    mutex: &'static SharedFlashMutex,
    slot_index: u8,
) -> Option<PointingSettingsSnapshot> {
    let addr = slot_addr(slot_index);
    let mut buf = [0u8; BLOB_LEN];
    {
        let mut guard = mutex.lock().await;
        if ReadNorFlash::read(&mut *guard, addr, &mut buf).await.is_err() {
            return None;
        }
    }
    PointingSettingsSnapshot::decode(&buf)
}
