mod mpu;
mod processing;
mod imu;

use std::{time::{Duration, Instant}};

use mpu::MPU;
use processing::*;
use imu::*;

use linux_embedded_hal::{I2cdev};
use i2cdev::linux::LinuxI2CError;

use crate::{mpu::Vector};


const MPU_ADDR: u8 = 0x68;
const PRINT_PERIOD_MS: u128 = 100;
const ORIENTATION_ALPHA: f32 = 0.15;



struct State {
  imu: IMU,

  display_orientation: Orientation,
  display_gyro: Vector,
  display_acc: Vector
}
 
fn main() -> Result<(), LinuxI2CError> {
  println!("running");

  let i2c = I2cdev::new("/dev/i2c-1")?;

  // let mut delay = Delay;
  let mut mpu = MPU::new(i2c);
  let mut imu = IMU::new(mpu)?;

  imu.update()?;
  let display_orientation = imu.orientation.expect("IMU was updated");
  let sample = imu.last_sample;

  let mut state = State {
      imu,
      display_orientation,
      display_gyro: sample.gyro,
      display_acc: sample.acc,
  };
  println!("\n");

  let mut last_time = Instant::now();

  loop {
    let now = Instant::now();

    let dt = now.duration_since(last_time).as_millis();
    state.imu.update()?;

    if dt >= PRINT_PERIOD_MS {
      print!("\x1B[4A");

      state.display_gyro.smooth(&state.imu.last_sample.gyro);
      state.display_acc.smooth(&state.imu.last_sample.acc);
      state.display_orientation.pitch += ORIENTATION_ALPHA * (state.imu.orientation.unwrap().pitch - state.display_orientation.pitch);
      state.display_orientation.roll += ORIENTATION_ALPHA * (state.imu.orientation.unwrap().roll - state.display_orientation.roll);


      println!("\rgyro: {:?}", state.display_gyro);
      println!("\racc: {:?}", state.display_acc);
      println!("\rpitch: {:?}", state.display_orientation.pitch);
      println!("\rroll: {:?}", state.display_orientation.roll);

      last_time = Instant::now();
    }

    std::thread::sleep(Duration::from_millis(PRINT_PERIOD_MS as u64));

  }
}
