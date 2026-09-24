use core::convert::Infallible;

use embassy_nrf::gpio::{Flex, Level, OutputDrive, Pull};
use embedded_hal::digital::{ErrorType, InputPin, OutputPin};
use rmk::driver::flex_pin::FlexPin;

/// Newtype wrapper: FlexPin impl for embassy_nrf::gpio::Flex (orphan rule workaround)
pub struct NrfFlex<'d>(pub Flex<'d>);

impl<'d> ErrorType for NrfFlex<'d> {
    type Error = Infallible;
}

impl<'d> InputPin for NrfFlex<'d> {
    fn is_high(&mut self) -> Result<bool, Infallible> {
        Ok(self.0.is_high())
    }
    fn is_low(&mut self) -> Result<bool, Infallible> {
        Ok(self.0.is_low())
    }
}

impl<'d> OutputPin for NrfFlex<'d> {
    fn set_high(&mut self) -> Result<(), Infallible> {
        self.0.set_high();
        Ok(())
    }
    fn set_low(&mut self) -> Result<(), Infallible> {
        self.0.set_low();
        Ok(())
    }
}

impl<'d> FlexPin for NrfFlex<'d> {
    fn set_as_input(&mut self) {
        self.0.set_as_input(Pull::None);
    }
    fn set_as_output(&mut self) {
        self.0.set_level(Level::Low);
        self.0.set_as_output(OutputDrive::Standard);
    }
}
