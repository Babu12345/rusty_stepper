//! Configure the stepper motor hardware

use crate::errors::Result;
use channel_v2::ChannelV2;
use esp_hal::{
    gpio::OutputPin,
    ledc::{
        channel::{self},
        timer::{TimerIFace, TimerSpeed},
        Ledc,
    },
    peripheral::Peripheral,
};
use stepper::stepper::Stepper;

/// Default PWM frequency
pub const FREQUENCY: u32 = 100;

/// Module around ChannelV2 newtype
pub mod channel_v2 {
    use crate::errors::{Error, Result};

    use esp_hal::{
        gpio::interconnect::PeripheralOutput,
        ledc::{
            channel::{self, Channel, ChannelIFace},
            timer::{TimerIFace, TimerSpeed},
            Ledc,
        },
        peripheral::Peripheral,
    };
    use stepper::stepper::SetPhaseAndDutyCycle;

    const MAX_HPOINT: u32 = 2u32.pow(14);
    const MAX_ANGLE_DEG: u32 = 360;
    const DUTY_CYCLE_PERCENTAGE: u8 = 50;

    /// Newtype around Channel with phase features
    pub struct ChannelV2<'channel, T: TimerSpeed> {
        channel: Channel<'channel, T>,
        channel_num: channel::Number,
    }

    impl<'channel, 'timer, T: TimerSpeed> ChannelV2<'channel, T> {
        /// Create new instance
        pub fn new<
            OUTPUT: Peripheral<P = impl PeripheralOutput> + 'channel,
            TIMER: TimerIFace<T>,
        >(
            ledc: &Ledc<'channel>,
            timer: &'channel TIMER,
            pin: OUTPUT,
            channel_num: channel::Number,
        ) -> Result<Self> {
            let mut channel = ledc.channel(channel_num, pin);
            channel
                .configure(channel::config::Config {
                    timer,
                    duty_pct: DUTY_CYCLE_PERCENTAGE,
                    pin_config: channel::config::PinConfig::PushPull,
                })
                .map_err(|_| Error::ChannelConfigurationError)?;

            Ok(Self {
                channel,
                channel_num,
            })
        }

        fn update_channel(&self) -> Result<()> {
            let ledc = unsafe { &*esp_hal::peripherals::LEDC::ptr() };
            let para_up = ledc
                .ch(self.channel_num as usize)
                .conf0()
                // Modify means that the other bits in the register will stay the same
                // and we can also read the contents of the register
                .modify(|_, w| w.para_up().set_bit());

            if (para_up >> 4 & 1) != 1 {
                return Err(Error::CustomError("Unable to update channel"));
            }
            Ok(())
        }
    }

    impl<'channel, T: TimerSpeed> SetPhaseAndDutyCycle for ChannelV2<'channel, T> {
        /// Set phase in degrees
        /// Use the PAC to set the phase offset on the device since the Hal doesn't yet have an implementation
        fn set_phase(&self, phase: u32) -> stepper::errors::Result<&Self> {
            assert!(phase <= 360, "Angle must be less than 360 degrees");
            let val = phase * MAX_HPOINT / MAX_ANGLE_DEG;

            let ledc = unsafe { &*esp_hal::peripherals::LEDC::ptr() };
            let hpoint = ledc
                .ch(self.channel_num as usize)
                .hpoint()
                .write(|w| unsafe { w.bits(val) });

            if hpoint != val {
                return Err(stepper::errors::Error::PhaseConfigurationError);
            }

            self.update_channel()
                .map_err(|_| stepper::errors::Error::PhaseConfigurationError)?;

            Ok(self)
        }

        /// Set the duty cycle
        fn set_duty(&self, duty_pct: u8) -> stepper::errors::Result<&Self> {
            self.channel
                .set_duty(duty_pct)
                .map_err(|_| stepper::errors::Error::DutyConfigurationError)?;
            Ok(self)
        }
    }
}

/// Configure the stepper motor with pins
pub fn configure_stepper<'stepper, T: TimerSpeed + 'stepper, TIMER: TimerIFace<T> + 'stepper>(
    ledc: Ledc<'stepper>,
    lstimer0: &'stepper TIMER,
    pin_a1: impl Peripheral<P = impl OutputPin> + 'stepper,
    pin_a2: impl Peripheral<P = impl OutputPin> + 'stepper,
    pin_b1: impl Peripheral<P = impl OutputPin> + 'stepper,
    pin_b2: impl Peripheral<P = impl OutputPin> + 'stepper,
) -> Result<Stepper<ChannelV2<'stepper, T>>> {
    let channel_a1 = ChannelV2::new(&ledc, lstimer0, pin_a1, channel::Number::Channel0)?;
    let channel_a2 = ChannelV2::new(&ledc, lstimer0, pin_a2, channel::Number::Channel1)?;
    let channel_b1 = ChannelV2::new(&ledc, lstimer0, pin_b1, channel::Number::Channel2)?;
    let channel_b2 = ChannelV2::new(&ledc, lstimer0, pin_b2, channel::Number::Channel3)?;

    Ok(Stepper::new(channel_a1, channel_a2, channel_b1, channel_b2))
}
