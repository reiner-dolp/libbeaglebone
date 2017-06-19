//! The PWM module.

use errors::*;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use util::*;

/// The state in which the PWM is in, either on or off.
#[derive(Debug, PartialEq, Eq)]
pub enum PWMState {
  /// PWM on
  Enabled,
  /// PWM off
  Disabled,
}

/// Represents a PWM device.
#[derive(Debug)]
pub struct PWM {
  pwm_chip_num: u8,
  pwm_num: u8,
  period: u32,
  duty_cycle: u32,
  state: PWMState,
}

impl PWM {
  /// Creates a new PWM object.
  ///
  /// Note: you will need to configure the selected pin as a PWM output prior
  /// to use using the `config-pin` utility.
  /// For example, `config-pin P9.21 pwm`.
  /// See the `examples/` directory for more help.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// use libbeaglebone::pwm::PWM;
  ///
  /// // Create a new PWM device using PWM chip 0 and PWM 0.
  /// let mut pwm = PWM::new(0,0);
  /// ```
  pub fn new(m_pwm_chip_num: u8, m_pwm_num: u8) -> PWM {
    PWM {
      pwm_chip_num: m_pwm_chip_num,
      pwm_num: m_pwm_num,
      period: 0,
      duty_cycle: 0,
      state: PWMState::Disabled,
    }
  }

  /// Exports the PWM.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// use libbeaglebone::pwm::PWM;
  ///
  /// // Create a new PWM device using PWM chip 0 and PWM 0.
  /// let mut pwm = PWM::new(0,0);
  ///
  /// // Export the PWM.
  /// pwm.set_export(true).unwrap();
  /// ```
  pub fn set_export(&self, state: bool) -> Result<()> {
    let path = PathBuf::from(format!("/sys/class/pwm/pwmchip{}/pwm{}",
                                     &self.pwm_chip_num,
                                     &self.pwm_num));
    if state && !path.exists() {
      File::create(format!("/sys/class/pwm/pwmchip{}/export", &self.pwm_chip_num))
        .chain_err(|| "Failed to open PWM export file")?
        .write_all(self.pwm_num.to_string().as_bytes())
        .chain_err(|| {
                     format!("Failed to export PWM #{}-{}",
                             &self.pwm_chip_num,
                             &self.pwm_num)
                   })?;
    }
    // Try to unexport if the path exists, otherwise the device is unexported and there's nothing
    // to do
    else if !state && path.exists() {
      File::create(format!("/sys/class/pwm/pwmchip{}/unexport", &self.pwm_chip_num))
        .chain_err(|| "Failed to open PWM unexport file")?
        .write_all(self.pwm_num.to_string().as_bytes())
        .chain_err(|| {
                     format!("Failed to unexport PWM #{}-{}",
                             &self.pwm_chip_num,
                             &self.pwm_num)
                   })?;
    }
    Ok(())
  }

  /// Sets the period of the PWM in nanoseconds.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// use libbeaglebone::pwm::PWM;
  ///
  /// // Create a new PWM device using PWM chip 0 and PWM 0.
  /// let mut pwm = PWM::new(0,0);
  ///
  /// // Export the PWM.
  /// pwm.set_export(true).unwrap();
  ///
  /// // Make the period 500,000ns.
  /// pwm.set_period(500_000).unwrap();
  /// ```
  pub fn set_period(&mut self, period_ns: u32) -> Result<()> {
    let path = format!("/sys/class/pwm/pwmchip{}/pwm{}/period",
                       &self.pwm_chip_num,
                       &self.pwm_num);
    write_file(&format!("{}", period_ns), &path)
      .chain_err(|| {
                   format!("Failed to set PWM #{}-{} period to {}",
                           &self.pwm_chip_num,
                           &self.pwm_num,
                           period_ns)
                 })?;
    self.period = period_ns;
    Ok(())
  }

  /// Sets the state (enabled or disabled) of the PWM.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// use libbeaglebone::pwm::{PWM, PWMState};
  ///
  /// // Create a new PWM device using PWM chip 0 and PWM 0.
  /// let mut pwm = PWM::new(0,0);
  ///
  /// // Export the PWM.
  /// pwm.set_export(true).unwrap();
  ///
  /// // Make the period 500,000ns.
  /// pwm.set_period(500_000).unwrap();
  ///
  /// // Turn the PWM on
  /// pwm.set_state(PWMState::Enabled).unwrap();
  /// ```
  pub fn set_state(&mut self, state: PWMState) -> Result<()> {
    let path = format!("/sys/class/pwm/pwmchip{}/pwm{}/enable",
                       &self.pwm_chip_num,
                       &self.pwm_num);
    write_file(match state {
                 PWMState::Enabled => "1",
                 PWMState::Disabled => "0",
               },
               &path)
      .chain_err(|| {
                   format!("Failed to set PWM #{}-{} state to {:?}",
                           &self.pwm_chip_num,
                           &self.pwm_num,
                           state)
                 })?;
    self.state = state;
    Ok(())
  }

  /// Sets the duty cycle of the PWM as a percentage of the period.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// use libbeaglebone::pwm::{PWM, PWMState};
  ///
  /// // Create a new PWM device using PWM chip 0 and PWM 0.
  /// let mut pwm = PWM::new(0,0);
  ///
  /// // Export the PWM.
  /// pwm.set_export(true).unwrap();
  ///
  /// // Make the period 500,000ns.
  /// pwm.set_period(500_000).unwrap();
  ///
  /// // Turn the PWM on.
  /// pwm.set_state(PWMState::Enabled).unwrap();
  ///
  /// // Set the duty cycle to 50% (250,000ns).
  /// pwm.write(50.0).unwrap();
  /// ```
  pub fn write(&mut self, percentage: f32) -> Result<()> {
    let path = format!("/sys/class/pwm/pwmchip{}/pwm{}/duty_cycle",
                       &self.pwm_chip_num,
                       &self.pwm_num);
    let new_duty_cycle = ((percentage / 100.0) * (self.period as f32)) as u32;
    write_file(&format!("{}", new_duty_cycle), &path)
      .chain_err(|| {
                   format!("Failed to set PWM #{}-{} duty cycle to {}% (aka {}ns)",
                           &self.pwm_chip_num,
                           &self.pwm_num,
                           percentage,
                           new_duty_cycle)
                 })?;
    self.duty_cycle = new_duty_cycle;
    Ok(())
  }

  /// Sets the duty cycle of the PWM in nanoseconds.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// use libbeaglebone::pwm::{PWM, PWMState};
  ///
  /// // Create a new PWM device using PWM chip 0 and PWM 0.
  /// let mut pwm = PWM::new(0,0);
  ///
  /// // Export the PWM.
  /// pwm.set_export(true).unwrap();
  ///
  /// // Make the period 500,000ns.
  /// pwm.set_period(500_000).unwrap();
  ///
  /// // Turn the PWM on.
  /// pwm.set_state(PWMState::Enabled).unwrap();
  ///
  /// // Set the duty cycle to 250,000ns.
  /// pwm.set_duty_cycle(250_000).unwrap();
  /// ```
  pub fn set_duty_cycle(&mut self, duty_cycle_ns: u32) -> Result<()> {
    let path = format!("/sys/class/pwm/pwmchip{}/pwm{}/duty_cycle",
                       &self.pwm_chip_num,
                       &self.pwm_num);
    write_file(&format!("{}", duty_cycle_ns), &path)
      .chain_err(|| {
                   format!("Failed to set PWM #{}-{} duty cycle to {}ns",
                           &self.pwm_chip_num,
                           &self.pwm_num,
                           duty_cycle_ns)
                 })?;
    self.duty_cycle = duty_cycle_ns;
    Ok(())
  }
}