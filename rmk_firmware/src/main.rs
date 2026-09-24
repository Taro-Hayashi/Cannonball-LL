#![no_std]
#![no_main]

mod keymap;
mod nrf_flex;
mod pointing_slots;
mod sleep;
mod vial;

use defmt::unwrap;
use defmt_rtt as _;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Flex, Input, Level, Output, OutputDrive, Pull};
use embassy_nrf::peripherals::{RNG, USBD};
use embassy_nrf::usb::Driver;
use embassy_nrf::usb::vbus_detect::HardwareVbusDetect;
use embassy_nrf::{bind_interrupts, pac, rng, usb};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::Mutex;
use keymap::{
    COL, ROW, SCROLL_LAYER, USER_CPI_DOWN, USER_CPI_UP, USER_CURSOR_TOGGLE, USER_LOAD_SLOT_1,
    USER_LOAD_SLOT_2, USER_SAVE_SLOT_1, USER_SAVE_SLOT_2, USER_SCROLL_INVERT_X,
    USER_SCROLL_INVERT_Y, USER_SCROLL_SPEED_DOWN, USER_SCROLL_SPEED_UP, USER_SNIPE,
};
use nrf_flex::NrfFlex;
use nrf_mpsl::Flash;
use pointing_slots::{
    PointingSettingsSnapshot, SharedFlash, SharedFlashMutex, load_slot, save_slot,
};
use nrf_sdc::mpsl::MultiprotocolServiceLayer;
use nrf_sdc::{self as sdc, mpsl};
use panic_probe as _;
use rand_chacha::ChaCha12Rng;
use rand_core::SeedableRng;
use rmk::ble::build_ble_stack;
use rmk::channel::USER_KEY_EVENT_CHANNEL;
use rmk::config::{
    BehaviorConfig, BleBatteryConfig, DeviceConfig, PositionalConfig, RmkConfig, StorageConfig,
    VialConfig,
};
use rmk::debounce::fast_debouncer::FastDebouncer;
use rmk::driver::bitbang_spi::BitBangSpiBus;
use rmk::event::{PointingSetCpiEvent, publish_event};
use rmk::futures::future::join5;
use rmk::input_device::Runnable;
use rmk::input_device::pmw3610::{Pmw3610, Pmw3610Config};
use rmk::input_device::pointing::{
    PointingDevice, PointingProcessor, PointingProcessorConfig, PointingRuntimeState,
    PointingRuntimeStateCell,
};
use rmk::input_device::rotary_encoder::RotaryEncoder;
use rmk::keyboard::Keyboard;
use rmk::matrix::hc595_matrix::Hc595Matrix;
use rmk::{HostResources, KeymapData, initialize_keymap_and_storage, run_all, run_rmk};
use static_cell::StaticCell;
use vial::{VIAL_KEYBOARD_DEF, VIAL_KEYBOARD_ID};

// --- Pointing tunables (7-step tables, center = firmware default) ---
const CPI_STEPS: [u16; 7] = [400, 600, 900, 1200, 1600, 2100, 2800];
const SCROLL_DIVISOR_STEPS: [u8; 7] = [8, 12, 18, 24, 36, 54, 72];
const DEFAULT_CPI_STEP: u8 = 3;
const DEFAULT_SCROLL_STEP: u8 = 3;
const PMW_DEVICE_ID: u8 = 0;

static POINTING_RUNTIME: PointingRuntimeStateCell = PointingRuntimeStateCell::new(PointingRuntimeState {
    cursor_enabled: true,
    scroll_divisor: SCROLL_DIVISOR_STEPS[DEFAULT_SCROLL_STEP as usize],
    scroll_invert_wheel: false,
    scroll_invert_pan: false,
});

bind_interrupts!(struct Irqs {
    USBD => usb::InterruptHandler<USBD>;
    RNG => rng::InterruptHandler<RNG>;
    EGU0_SWI0 => nrf_sdc::mpsl::LowPrioInterruptHandler;
    CLOCK_POWER => nrf_sdc::mpsl::ClockInterruptHandler, usb::vbus_detect::InterruptHandler;
    RADIO => nrf_sdc::mpsl::HighPrioInterruptHandler;
    TIMER0 => nrf_sdc::mpsl::HighPrioInterruptHandler;
    RTC0 => nrf_sdc::mpsl::HighPrioInterruptHandler;
});

const UNLOCK_KEYS: &[(u8, u8)] = &[(0, 0), (0, 1)];

const L2CAP_MTU: usize = 251;
const L2CAP_TXQ: u8 = 3;
const L2CAP_RXQ: u8 = 3;

struct DummyCs;

impl embedded_hal::digital::ErrorType for DummyCs {
    type Error = core::convert::Infallible;
}

impl embedded_hal::digital::OutputPin for DummyCs {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[embassy_executor::task]
async fn mpsl_task(mpsl: &'static MultiprotocolServiceLayer<'static>) -> ! {
    mpsl.run().await
}

fn build_sdc<'d, const N: usize>(
    p: nrf_sdc::Peripherals<'d>,
    rng: &'d mut rng::Rng<'d, embassy_nrf::mode::Async>,
    mpsl: &'d MultiprotocolServiceLayer,
    mem: &'d mut sdc::Mem<N>,
) -> Result<nrf_sdc::SoftdeviceController<'d>, nrf_sdc::Error> {
    sdc::Builder::new()?
        .support_adv()
        .support_peripheral()
        .support_dle_peripheral()
        .support_phy_update_peripheral()
        .support_le_2m_phy()
        .peripheral_count(1)?
        .buffer_cfg(L2CAP_MTU as u16, L2CAP_MTU as u16, L2CAP_TXQ, L2CAP_RXQ)?
        .build(p, rng, mpsl, mem)
}

fn ble_addr() -> [u8; 6] {
    let ficr = pac::FICR;
    let high = u64::from(ficr.deviceid(1).read());
    let addr = high << 32 | u64::from(ficr.deviceid(0).read());
    let addr = addr | 0x0000_c000_0000_0000;
    unwrap!(addr.to_le_bytes()[..6].try_into())
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // nRF config with DCDC
    let mut nrf_config = embassy_nrf::config::Config::default();
    nrf_config.dcdc.reg0_voltage = Some(embassy_nrf::config::Reg0Voltage::_3V3);
    nrf_config.dcdc.reg0 = true;
    nrf_config.dcdc.reg1 = true;
    let p = embassy_nrf::init(nrf_config);

    // --- MPSL (Multiprotocol Service Layer) ---
    let mpsl_p =
        mpsl::Peripherals::new(p.RTC0, p.TIMER0, p.TEMP, p.PPI_CH19, p.PPI_CH30, p.PPI_CH31);
    let lfclk_cfg = mpsl::raw::mpsl_clock_lfclk_cfg_t {
        source: mpsl::raw::MPSL_CLOCK_LF_SRC_RC as u8,
        rc_ctiv: mpsl::raw::MPSL_RECOMMENDED_RC_CTIV as u8,
        rc_temp_ctiv: mpsl::raw::MPSL_RECOMMENDED_RC_TEMP_CTIV as u8,
        accuracy_ppm: mpsl::raw::MPSL_DEFAULT_CLOCK_ACCURACY_PPM as u16,
        skip_wait_lfclk_started: mpsl::raw::MPSL_DEFAULT_SKIP_WAIT_LFCLK_STARTED != 0,
    };
    static MPSL: StaticCell<MultiprotocolServiceLayer> = StaticCell::new();
    static SESSION_MEM: StaticCell<mpsl::SessionMem<1>> = StaticCell::new();
    let mpsl = MPSL.init(unwrap!(mpsl::MultiprotocolServiceLayer::with_timeslots(
        mpsl_p,
        Irqs,
        lfclk_cfg,
        SESSION_MEM.init(mpsl::SessionMem::new()),
    )));
    spawner.spawn(mpsl_task(&*mpsl).unwrap());

    // --- SDC (SoftDevice Controller) ---
    let sdc_p = sdc::Peripherals::new(
        p.PPI_CH17, p.PPI_CH18, p.PPI_CH20, p.PPI_CH21, p.PPI_CH22, p.PPI_CH23, p.PPI_CH24,
        p.PPI_CH25, p.PPI_CH26, p.PPI_CH27, p.PPI_CH28, p.PPI_CH29,
    );
    static RNG_INST: StaticCell<rng::Rng<'static, embassy_nrf::mode::Async>> = StaticCell::new();
    let rng_inst = RNG_INST.init(rng::Rng::new(p.RNG, Irqs));
    let mut rng_gen = ChaCha12Rng::from_rng(&mut *rng_inst).unwrap();
    static SDC_MEM: StaticCell<sdc::Mem<4096>> = StaticCell::new();
    let sdc_mem = SDC_MEM.init(sdc::Mem::<4096>::new());
    let sdc = unwrap!(build_sdc(sdc_p, rng_inst, mpsl, sdc_mem));

    // --- BLE stack ---
    let mut host_resources = HostResources::new();
    let stack = build_ble_stack(sdc, ble_addr(), &mut rng_gen, &mut host_resources).await;

    // USB driver
    let driver = Driver::new(p.USBD, Irqs, HardwareVbusDetect::new(Irqs));

    // Internal flash (MPSL-aware) shared between rmk storage and slot I/O.
    static FLASH_MUTEX: StaticCell<SharedFlashMutex> = StaticCell::new();
    let flash_mutex: &'static SharedFlashMutex =
        FLASH_MUTEX.init(Mutex::new(Flash::take(mpsl, p.NVMC)));
    let flash = SharedFlash::new(flash_mutex);

    // --- 74HC595 matrix ---
    let row_pins = [
        Input::new(p.P0_28, Pull::Down),
        Input::new(p.P0_29, Pull::Down),
    ];
    let debouncer = FastDebouncer::new();
    let matrix_sck = Output::new(p.P0_05, Level::High, OutputDrive::Standard);
    let matrix_sdio = NrfFlex(Flex::new(p.P0_04));
    let shared_bus = {
        static SPI_BUS: StaticCell<
            Mutex<ThreadModeRawMutex, BitBangSpiBus<Output<'static>, NrfFlex<'static>>>,
        > = StaticCell::new();
        SPI_BUS.init(Mutex::new(BitBangSpiBus::new(matrix_sck, matrix_sdio)))
    };
    let matrix_spi = SpiDevice::new(shared_bus, DummyCs);
    let matrix_cs = Output::new(p.P1_11, Level::High, OutputDrive::Standard);
    let mut matrix = Hc595Matrix::<_, _, _, _, ROW, COL>::new(matrix_spi, matrix_cs, row_pins, debouncer).await;

    // --- PMW3610 trackball (shares the matrix SPI bus) ---
    let pmw_cs = Output::new(p.P0_10, Level::High, OutputDrive::Standard);
    let pmw_spi = SpiDevice::new(shared_bus, pmw_cs);
    let pmw_motion = Input::new(p.P0_09, Pull::Up);
    let pmw_config = Pmw3610Config {
        res_cpi: 1200,
        swap_xy: true,
        invert_x: !cfg!(feature = "sensor-rotated-180"),
        invert_y: cfg!(feature = "sensor-rotated-180"),
        ..Default::default()
    };
    let mut pointing_device =
        PointingDevice::<Pmw3610<_, _>>::new(0, pmw_spi, Some(pmw_motion), pmw_config);

    // --- Rotary encoders ---
    let mut enc_head = RotaryEncoder::new(
        Input::new(p.P0_03, Pull::Up),
        Input::new(p.P0_02, Pull::Up),
        0,
    );
    let mut enc_chest = RotaryEncoder::new(
        Input::new(p.P1_15, Pull::Up),
        Input::new(p.P1_14, Pull::Up),
        1,
    );
    #[cfg(not(feature = "sensor-rotated-180"))]
    let mut enc_leg = RotaryEncoder::new(
        Input::new(p.P1_13, Pull::Up),
        Input::new(p.P1_12, Pull::Up),
        2,
    );
    #[cfg(feature = "sensor-rotated-180")]
    let mut enc_leg = RotaryEncoder::new(
        Input::new(p.P1_12, Pull::Up),
        Input::new(p.P1_13, Pull::Up),
        2,
    );

    // --- RMK config ---
    let storage_config = StorageConfig {
        start_addr: 0xA0000,
        num_sectors: 6,
        clear_storage: false,
        ..Default::default()
    };
    let ble_battery_config = BleBatteryConfig::default();
    let rmk_config = RmkConfig {
        device_config: DeviceConfig {
            vid: 0x8884,
            pid: 0x0919,
            manufacturer: "Taro Hayashi",
            product_name: "Cannonball LL",
            serial_number: "vial:f64c2b3c:000001",
        },
        vial_config: VialConfig::new(VIAL_KEYBOARD_ID, VIAL_KEYBOARD_DEF, UNLOCK_KEYS),
        storage_config,
        ble_battery_config,
        ..Default::default()
    };

    let mut keymap_data = KeymapData::new_with_encoder(
        keymap::get_default_keymap(),
        keymap::get_default_encoder_map(),
    );
    let mut behavior_config = BehaviorConfig::default();
    let per_key_config = PositionalConfig::default();
    let (keymap, mut storage) = initialize_keymap_and_storage(
        &mut keymap_data,
        flash,
        &rmk_config.storage_config,
        &mut behavior_config,
        &per_key_config,
    )
    .await;

    let mut keyboard = Keyboard::new(&keymap);
    let mut pointing_processor = PointingProcessor::new(
        &keymap,
        PointingProcessorConfig {
            scroll_layer: Some(SCROLL_LAYER),
            ..Default::default()
        },
    )
    .with_runtime_state(&POINTING_RUNTIME);

    // Publish the initial CPI so PMW3610 matches the firmware default step.
    publish_event(PointingSetCpiEvent {
        device_id: PMW_DEVICE_ID,
        cpi: CPI_STEPS[DEFAULT_CPI_STEP as usize],
    });

    join5(
        run_all!(matrix, enc_head, enc_chest, enc_leg, pointing_device, pointing_processor),
        keyboard.run(),
        run_rmk(&keymap, driver, &stack, &mut storage, rmk_config),
        pointing_user_key_dispatcher(flash_mutex),
        sleep::sleep_manager(),
    )
    .await;
}

fn set_cpi(step: u8) {
    let cpi = CPI_STEPS[step as usize];
    publish_event(PointingSetCpiEvent { device_id: PMW_DEVICE_ID, cpi });
    defmt::info!("CPI step {} ({} cpi)", step, cpi);
}

fn apply_snapshot(snapshot: PointingSettingsSnapshot) -> Option<(u8, u8)> {
    if snapshot.cpi_step as usize >= CPI_STEPS.len()
        || snapshot.scroll_step as usize >= SCROLL_DIVISOR_STEPS.len()
    {
        return None;
    }
    let divisor = SCROLL_DIVISOR_STEPS[snapshot.scroll_step as usize];
    POINTING_RUNTIME.update(|s| {
        s.cursor_enabled = snapshot.cursor_enabled;
        s.scroll_divisor = divisor;
        s.scroll_invert_wheel = snapshot.scroll_invert_wheel;
        s.scroll_invert_pan = snapshot.scroll_invert_pan;
    });
    set_cpi(snapshot.cpi_step);
    Some((snapshot.cpi_step, snapshot.scroll_step))
}

/// Listens on USER_KEY_EVENT_CHANNEL and mutates pointing runtime state
/// + republishes PMW3610 CPI changes. Snipe saves the active CPI step on
/// press and restores it on release. On startup, loads Slot 1 if present.
async fn pointing_user_key_dispatcher(flash_mutex: &'static SharedFlashMutex) {
    let mut cpi_step: u8 = DEFAULT_CPI_STEP;
    let mut scroll_step: u8 = DEFAULT_SCROLL_STEP;
    let mut snipe_saved_step: Option<u8> = None;

    // Wait for PointingDevice to subscribe to PointingSetCpiEvent before
    // publishing — publish_immediate drops messages when no subscriber exists.
    embassy_time::Timer::after(embassy_time::Duration::from_millis(500)).await;

    // Auto-load Slot 1 on boot; fall back to defaults if absent/invalid.
    let loaded = match load_slot(flash_mutex, 1).await {
        Some(snapshot) => match apply_snapshot(snapshot) {
            Some((c, s)) => {
                cpi_step = c;
                scroll_step = s;
                defmt::info!("Loaded pointing Slot 1 on boot");
                true
            }
            None => {
                defmt::warn!("Slot 1 blob rejected (out-of-range step)");
                false
            }
        },
        None => false,
    };
    if !loaded {
        set_cpi(cpi_step);
    }

    loop {
        let evt = USER_KEY_EVENT_CHANNEL.receive().await;
        match (evt.id, evt.pressed) {
            (USER_CPI_UP, true) => {
                if cpi_step + 1 < CPI_STEPS.len() as u8 {
                    cpi_step += 1;
                    set_cpi(cpi_step);
                }
            }
            (USER_CPI_DOWN, true) => {
                if cpi_step > 0 {
                    cpi_step -= 1;
                    set_cpi(cpi_step);
                }
            }
            (USER_CURSOR_TOGGLE, true) => {
                POINTING_RUNTIME.update(|s| s.cursor_enabled = !s.cursor_enabled);
                let enabled = POINTING_RUNTIME.get().cursor_enabled;
                defmt::info!("Cursor enabled: {}", enabled);
            }
            (USER_SNIPE, true) => {
                if snipe_saved_step.is_none() {
                    snipe_saved_step = Some(cpi_step);
                    cpi_step = 0;
                    set_cpi(cpi_step);
                }
            }
            (USER_SNIPE, false) => {
                if let Some(saved) = snipe_saved_step.take() {
                    cpi_step = saved;
                    set_cpi(cpi_step);
                }
            }
            (USER_SCROLL_SPEED_UP, true) => {
                if scroll_step > 0 {
                    scroll_step -= 1;
                    let divisor = SCROLL_DIVISOR_STEPS[scroll_step as usize];
                    POINTING_RUNTIME.update(|s| s.scroll_divisor = divisor);
                    defmt::info!("Scroll step {} (divisor {})", scroll_step, divisor);
                }
            }
            (USER_SCROLL_SPEED_DOWN, true) => {
                if scroll_step + 1 < SCROLL_DIVISOR_STEPS.len() as u8 {
                    scroll_step += 1;
                    let divisor = SCROLL_DIVISOR_STEPS[scroll_step as usize];
                    POINTING_RUNTIME.update(|s| s.scroll_divisor = divisor);
                    defmt::info!("Scroll step {} (divisor {})", scroll_step, divisor);
                }
            }
            (USER_SCROLL_INVERT_X, true) => {
                POINTING_RUNTIME.update(|s| s.scroll_invert_pan = !s.scroll_invert_pan);
                defmt::info!("Scroll invert X: {}", POINTING_RUNTIME.get().scroll_invert_pan);
            }
            (USER_SCROLL_INVERT_Y, true) => {
                POINTING_RUNTIME.update(|s| s.scroll_invert_wheel = !s.scroll_invert_wheel);
                defmt::info!("Scroll invert Y: {}", POINTING_RUNTIME.get().scroll_invert_wheel);
            }
            (USER_SAVE_SLOT_1, true) => {
                save_current(flash_mutex, 1, cpi_step, scroll_step).await;
            }
            (USER_LOAD_SLOT_1, true) => {
                if let Some((c, s)) = load_and_apply(flash_mutex, 1).await {
                    cpi_step = c;
                    scroll_step = s;
                }
            }
            (USER_SAVE_SLOT_2, true) => {
                save_current(flash_mutex, 2, cpi_step, scroll_step).await;
            }
            (USER_LOAD_SLOT_2, true) => {
                if let Some((c, s)) = load_and_apply(flash_mutex, 2).await {
                    cpi_step = c;
                    scroll_step = s;
                }
            }
            _ => {}
        }
    }
}

async fn save_current(
    flash_mutex: &'static SharedFlashMutex,
    slot: u8,
    cpi_step: u8,
    scroll_step: u8,
) {
    let rt = POINTING_RUNTIME.get();
    let snapshot = PointingSettingsSnapshot {
        cpi_step,
        scroll_step,
        cursor_enabled: rt.cursor_enabled,
        scroll_invert_wheel: rt.scroll_invert_wheel,
        scroll_invert_pan: rt.scroll_invert_pan,
    };
    match save_slot(flash_mutex, slot, snapshot).await {
        Ok(()) => defmt::info!("Saved pointing Slot {}", slot),
        Err(_) => defmt::error!("Save Slot {} failed", slot),
    }
}

async fn load_and_apply(flash_mutex: &'static SharedFlashMutex, slot: u8) -> Option<(u8, u8)> {
    match load_slot(flash_mutex, slot).await {
        Some(snapshot) => match apply_snapshot(snapshot) {
            Some(pair) => {
                defmt::info!("Loaded pointing Slot {}", slot);
                Some(pair)
            }
            None => {
                defmt::warn!("Slot {} blob rejected (out-of-range step)", slot);
                None
            }
        },
        None => {
            defmt::warn!("Slot {} empty or invalid", slot);
            None
        }
    }
}
