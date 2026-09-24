//! Deep sleep (System OFF) after a period of inactivity.
//!
//! Any key, encoder or trackball activity resets the idle timer. When it expires
//! (and USB power is absent), wake sources are armed via GPIO SENSE and the chip
//! enters System OFF. Waking resets the chip, so the firmware boots and reconnects.

use embassy_nrf::pac;
use embassy_nrf::pac::gpio::vals::{Dir, Input, Pull, Sense};
use embassy_time::{Duration, Timer};
use rmk::channel::INPUT_ACTIVITY_SIGNAL;
use rmk::embassy_futures::select::{Either, select};

const IDLE_TIMEOUT: Duration = Duration::from_secs(30 * 60);

// 74HC595 column driver (bit-banged, shared with PMW3610)
const SR_SCK: (u8, usize) = (0, 5);
const SR_SDIO: (u8, usize) = (0, 4);
const SR_LATCH: (u8, usize) = (1, 11);
const SR_BITS: usize = 16; // 15 columns → two chained 595s
const PMW_CS: (u8, usize) = (0, 10);

const ROW_PINS: [(u8, usize); 2] = [(0, 28), (0, 29)];
const PMW_MOTION: (u8, usize) = (0, 9);
const ENCODER_PINS: [(u8, usize); 6] = [(0, 3), (0, 2), (1, 15), (1, 14), (1, 13), (1, 12)];

pub async fn sleep_manager() {
    loop {
        if let Either::Second(_) =
            select(INPUT_ACTIVITY_SIGNAL.wait(), Timer::after(IDLE_TIMEOUT)).await
        {
            if pac::POWER.usbregstatus().read().vbusdetect() {
                continue;
            }
            defmt::info!("Idle for {} s, entering System OFF", IDLE_TIMEOUT.as_secs());
            enter_system_off();
        }
    }
}

fn port(p: u8) -> pac::gpio::Gpio {
    if p == 0 { pac::P0 } else { pac::P1 }
}

fn set_output(pin: (u8, usize), high: bool) {
    let (p, n) = pin;
    if high {
        port(p).outset().write(|w| w.set_pin(n, true));
    } else {
        port(p).outclr().write(|w| w.set_pin(n, true));
    }
    port(p).dirset().write(|w| w.set_pin(n, true));
}

fn is_high(pin: (u8, usize)) -> bool {
    port(pin.0).in_().read().pin(pin.1)
}

fn arm_sense(pin: (u8, usize), pull: Pull, sense: Sense) {
    port(pin.0).pin_cnf(pin.1).write(|w| {
        w.set_dir(Dir::INPUT);
        w.set_input(Input::CONNECT);
        w.set_pull(pull);
        w.set_sense(sense);
    });
}

/// Drive every 595 output high so that any pressed key pulls its row high.
fn drive_all_columns_high() {
    const HALF_PERIOD: u32 = 64; // ~1 µs at 64 MHz
    set_output(PMW_CS, true); // keep the sensor off the shared bus
    set_output(SR_SDIO, true);
    set_output(SR_SCK, true);
    set_output(SR_LATCH, true);
    for _ in 0..SR_BITS {
        set_output(SR_SCK, false);
        cortex_m::asm::delay(HALF_PERIOD);
        set_output(SR_SCK, true);
        cortex_m::asm::delay(HALF_PERIOD);
    }
    set_output(SR_LATCH, false);
    cortex_m::asm::delay(HALF_PERIOD);
    set_output(SR_LATCH, true);
    cortex_m::asm::delay(HALF_PERIOD * 80); // let the rows settle
}

fn enter_system_off() -> ! {
    drive_all_columns_high();

    for pin in ROW_PINS {
        arm_sense(pin, Pull::PULLDOWN, Sense::HIGH);
    }

    // MOTION is active low and stays asserted until the motion registers are read;
    // arming it while asserted would wake us immediately.
    let motion_sense = if is_high(PMW_MOTION) {
        Sense::LOW
    } else {
        Sense::DISABLED
    };
    arm_sense(PMW_MOTION, Pull::PULLUP, motion_sense);

    // Encoder contacts rest either open or closed, so wake on any change from now.
    for pin in ENCODER_PINS {
        let sense = if is_high(pin) {
            Sense::LOW
        } else {
            Sense::HIGH
        };
        arm_sense(pin, Pull::PULLUP, sense);
    }

    // Clear stale latched detections so DETECT only reflects the armed pins.
    pac::P0
        .latch()
        .write_value(pac::gpio::regs::Latch(u32::MAX));
    pac::P1
        .latch()
        .write_value(pac::gpio::regs::Latch(u32::MAX));

    pac::POWER.systemoff().write(|w| w.set_systemoff(true));
    loop {
        cortex_m::asm::wfe();
    }
}
