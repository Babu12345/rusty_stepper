//! Hardware agnostic driver code for the bi-polar stepper motor

use crate::errors::Result;
use embassy_time::{Duration, Timer};

/// Trait for setting the phase and duty cycle
pub trait SetPhaseAndDutyCycle {
    /// Sets the phase of the pin. 0<=phase<=360
    fn set_phase(&self, phase: u32) -> Result<&Self>;
    /// Sets the duty cycle of the pin. 0<=duty_pct<=100
    fn set_duty(&self, duty_pct: u8) -> Result<&Self>;
}

/// Input to the drive function
pub struct Stepper<CHANNEL: SetPhaseAndDutyCycle> {
    // Coil Output A
    /// Pin A1
    channel_a1: CHANNEL,
    /// Pin A2
    channel_a2: CHANNEL,
    // // Coil Output B
    /// Pin B1
    channel_b1: CHANNEL,
    /// Pin B2
    channel_b2: CHANNEL,
}

/// Direction to drive the stepper
/// CW and CCW are both relative so set the pins appropriately
pub enum DIRECTION {
    /// Clockwise
    CW,
    /// Counter clockwise
    CCW,
    /// No motion
    OFF,
}

impl<CHANNEL: SetPhaseAndDutyCycle> Stepper<CHANNEL> {
    /// Contruct a new stepper
    pub fn new(
        channel_a1: CHANNEL,
        channel_a2: CHANNEL,
        channel_b1: CHANNEL,
        channel_b2: CHANNEL,
    ) -> Self {
        Self {
            channel_a1,
            channel_a2,
            channel_b1,
            channel_b2,
        }
    }
    /// Drive the stepper in a particular direction
    /// For CW/CCW states:
    ///     For each pair A1/A2 and B1/B2 they need to be 180 degrees out of phase
    ///     Between the 2 pairs there nees to be a 90 phase difference to get max torque.
    /// For off states:
    ///     Phases should be equal so no current can flow through the coil
    pub fn drive(&self, direction: DIRECTION) -> Result<&Self> {
        match direction {
            DIRECTION::CW => {
                self.channel_a1.set_phase(0)?;
                self.channel_a2.set_phase(180)?;
                self.channel_b1.set_phase(90)?;
                self.channel_b2.set_phase(180 + 90)?;
            }
            DIRECTION::CCW => {
                self.channel_a1.set_phase(90)?;
                self.channel_a2.set_phase(180 + 90)?;
                self.channel_b1.set_phase(0)?;
                self.channel_b2.set_phase(180)?;
            }
            DIRECTION::OFF => {
                self.channel_a1.set_phase(0)?;
                self.channel_a2.set_phase(0)?;
                self.channel_b1.set_phase(90)?;
                self.channel_b2.set_phase(90)?;
            }
        }

        Ok(self)
    }

    /// Drives for a particular duration
    pub async fn drive_for(&self, direction: DIRECTION, duration: Duration) -> Result<&Self> {
        self.drive(direction)?;
        Timer::after(duration).await;
        self.drive(DIRECTION::OFF)
    }
}
