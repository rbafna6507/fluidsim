mod mpu;
mod processing;

use std::{any::Any, thread, time::Duration};

use mpu::MPU;
use processing::Processor;

use linux_embedded_hal::{I2cdev, Delay};
use i2cdev::linux::LinuxI2CError;

use crate::{mpu::Vector, processing::Bias};

pub struct Orientation {
  pub pitch: f32,
  pub roll: f32
}

pub struct IMU {
  mpu: MPU,
  biases: Bias,
  gyro: Vector,
  acc: Vector,
  orientation: Orientation
}


const MPU_ADDR: u8 = 0x68;
 
fn main() -> Result<(), LinuxI2CError> {
  println!("running");

  let i2c = I2cdev::new("/dev/i2c-1")?;

  // let mut delay = Delay;
  let mut mpu = MPU::new(i2c);

  let processor = Processor {};

  mpu.init()?;


  let mut gyro = Vector::new();
  let mut acc = Vector::new();

  let bias = processor.calibrate_sensor_biases(&mut mpu)?;

  println!("\n");

  loop {
    print!("\x1B[2A");

    let raw_gyro = mpu.get_gyro()?;
    let raw_acc = mpu.get_acc()?;

    // let mpu_acc = Vector {x: raw_acc.x, y: raw_acc.y, z: 0.0} - calibration.acc;
    // let mpu_gyro = Vector {x: raw_gyro.x, y: raw_gyro.y, z: raw_gyro.z} - calibration.gyro;

    gyro.smooth(&(raw_gyro - bias.gyro));
    acc.smooth(&(raw_acc - bias.acc));

    println!("\rgyro: {:?}", gyro);
    println!("\racc: {:?}", acc);

    std::thread::sleep(Duration::from_millis(100));

  }
}
