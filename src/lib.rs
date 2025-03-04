#![no_std]

use bitflags::bitflags;
use embedded_hal::{
    delay::DelayNs,
    digital::{OutputPin, PinState},
};

bitflags! {
    pub struct MixerSettings: u8 {
        const DisableToneA = 0b00000001;
        const DisableToneB = 0b00000010;
        const DisableToneC = 0b00000100;
        const DisableNoiseA = 0b00001000;
        const DisableNoiseB = 0b00010000;
        const DisableNoiseC = 0b00100000;
        const OutputIOA = 0b01000000;
        const OutputIOB= 0b10000000;
    }
}

bitflags! {
    pub struct EnvelopeShape: u8 {
        const Hold = 0b0001;
        const Alt = 0b0010;
        const Att = 0b0100;
        const cont = 0b1000;
    }
}

pub enum Channel {
    A,
    B,
    C,
}

pub enum ChannelLevel {
    Fixed(u8),
    Envelope,
}

pub enum IoPort {
    A,
    B,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Error<P: OutputPin> {
    PinError(P::Error),
}

pub struct Ym2149<P, Delay> {
    bdir: P,
    bc1: P,
    d0: P,
    d1: P,
    d2: P,
    d3: P,
    d4: P,
    d5: P,
    d6: P,
    d7: P,
    delay: Delay,
}

impl<P, Delay> Ym2149<P, Delay>
where
    P: OutputPin,
    Delay: DelayNs,
{
    pub fn new(
        bdir: P,
        bc1: P,
        d0: P,
        d1: P,
        d2: P,
        d3: P,
        d4: P,
        d5: P,
        d6: P,
        d7: P,
        delay: Delay,
    ) -> Result<Ym2149<P, Delay>, Error<P>> {
        // TODO: Return pins if this fails
        let mut output = Ym2149 {
            bdir,
            bc1,
            d0,
            d1,
            d2,
            d3,
            d4,
            d5,
            d6,
            d7,
            delay,
        };
        output.inactive_mode()?;
        Ok(output)
    }

    fn write_mode(&mut self) -> Result<(), Error<P>> {
        self.bdir.set_high().map_err(Error::PinError)?;
        self.bc1.set_low().map_err(Error::PinError)?;
        Ok(())
    }

    fn address_mode(&mut self) -> Result<(), Error<P>> {
        self.bdir.set_high().map_err(Error::PinError)?;
        self.bc1.set_high().map_err(Error::PinError)?;
        Ok(())
    }

    fn inactive_mode(&mut self) -> Result<(), Error<P>> {
        self.bdir.set_low().map_err(Error::PinError)?;
        self.bc1.set_low().map_err(Error::PinError)?;
        Ok(())
    }

    fn write_u8(&mut self, data: u8) -> Result<(), Error<P>> {
        self.d0
            .set_state(if data & 0x01 == 1 {
                PinState::High
            } else {
                PinState::Low
            })
            .map_err(Error::PinError)?;
        self.d1
            .set_state(if (data >> 1) & 0x01 == 1 {
                PinState::High
            } else {
                PinState::Low
            })
            .map_err(Error::PinError)?;
        self.d2
            .set_state(if (data >> 2) & 0x01 == 1 {
                PinState::High
            } else {
                PinState::Low
            })
            .map_err(Error::PinError)?;
        self.d3
            .set_state(if (data >> 3) & 0x01 == 1 {
                PinState::High
            } else {
                PinState::Low
            })
            .map_err(Error::PinError)?;
        self.d4
            .set_state(if (data >> 4) & 0x01 == 1 {
                PinState::High
            } else {
                PinState::Low
            })
            .map_err(Error::PinError)?;
        self.d5
            .set_state(if (data >> 5) & 0x01 == 1 {
                PinState::High
            } else {
                PinState::Low
            })
            .map_err(Error::PinError)?;
        self.d6
            .set_state(if (data >> 6) & 0x01 == 1 {
                PinState::High
            } else {
                PinState::Low
            })
            .map_err(Error::PinError)?;
        self.d7
            .set_state(if (data >> 7) & 0x01 == 1 {
                PinState::High
            } else {
                PinState::Low
            })
            .map_err(Error::PinError)?;
        Ok(())
    }

    fn set_address(&mut self, address: u8) -> Result<(), Error<P>> {
        self.address_mode()?;
        self.write_u8(address)?;
        self.delay.delay_us(1);
        self.inactive_mode()?;
        self.delay.delay_us(1);
        Ok(())
    }

    fn set_data(&mut self, data: u8) -> Result<(), Error<P>> {
        self.write_u8(data)?;
        self.write_mode()?;
        self.delay.delay_us(1);
        self.inactive_mode()?;
        self.delay.delay_us(1);
        Ok(())
    }

    pub fn clear_all_registers(&mut self) -> Result<(), Error<P>> {
        for i in 0..16 {
            self.set_register_value(i, 0)?;
        }
        Ok(())
    }

    pub fn set_register_value(&mut self, address: u8, data: u8) -> Result<(), Error<P>> {
        self.set_address(address)?;
        self.set_data(data)?;
        Ok(())
    }

    pub fn set_channel_frequency(
        &mut self,
        channel: Channel,
        frequency: u16,
    ) -> Result<(), Error<P>> {
        let (fine_channel, rough_channel) = match channel {
            Channel::A => (0x0, 0x1),
            Channel::B => (0x2, 0x3),
            Channel::C => (0x4, 0x5),
        };
        let fine = frequency as u8;
        let rough = (frequency >> 8) as u8;
        self.set_register_value(fine_channel, fine)?;
        self.set_register_value(rough_channel, rough)?;
        Ok(())
    }

    pub fn set_noise(&mut self, frequency: u8) -> Result<(), Error<P>> {
        self.set_register_value(0x6, frequency)?;
        Ok(())
    }

    pub fn set_mixer_settings(&mut self, settings: MixerSettings) -> Result<(), Error<P>> {
        self.set_register_value(0x7, settings.bits())?;
        Ok(())
    }

    pub fn set_channel_level(
        &mut self,
        channel: Channel,
        level: ChannelLevel,
    ) -> Result<(), Error<P>> {
        let data = match level {
            ChannelLevel::Fixed(level) => level & 0b1111,
            ChannelLevel::Envelope => 0b10000,
        };
        let register = match channel {
            Channel::A => 0x8,
            Channel::B => 0x9,
            Channel::C => 0xA,
        };
        self.set_register_value(register, data)?;
        Ok(())
    }

    pub fn set_envelope_frequency(&mut self, frequency: u16) -> Result<(), Error<P>> {
        self.set_register_value(0xB, frequency as u8)?;
        self.set_register_value(0xC, (frequency >> 8) as u8)?;
        Ok(())
    }

    pub fn set_envelope_shape(&mut self, shape: EnvelopeShape) -> Result<(), Error<P>> {
        self.set_register_value(0xD, shape.bits())?;
        Ok(())
    }

    pub fn set_io_port_data(&mut self, port: IoPort, data: u8) -> Result<(), Error<P>> {
        let register = match port {
            IoPort::A => 0xE,
            IoPort::B => 0xD,
        };
        self.set_register_value(register, data)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        //let result = add(2, 2);
        assert_eq!(0, 4);
    }
}
