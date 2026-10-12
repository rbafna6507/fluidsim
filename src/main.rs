mod mpu;

use std::{any::Any, thread, time::Duration};

use mpu6050::*;
use std::ops::Sub;
use std::ops::Mul;
use std::ops::Div;
use linux_embedded_hal::{I2cdev, Delay};
use i2cdev::linux::LinuxI2CError;




const CALIBRATION_SAMPLES: usize = 500;
const SAMPLE_INTERVAL_MS: u64 = 5;

const ALPHA: f32 = 0.15;


struct Calibration {
  gyro: Vector,
  acc: Vector
}

 #[derive(Debug, Clone, Copy)]
 struct Vector {
  x: f32,
  y: f32,
  z: f32
}

impl Vector {

  fn new() -> Vector {
    Vector { x: 0.0, y: 0.0, z: 0.0 }
  }

  fn smooth(&mut self, new: & Vector) {
    
    self.x += ALPHA * (new.x - self.x);
    self.y += ALPHA * (new.y - self.y);
    self.z += ALPHA * (new.z - self.z);

  }

  fn magnitude(&self) -> f32 {
    (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
  }

}

impl Sub for Vector {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl Mul<f32> for Vector {
  type Output = Self;

  fn mul(self, other: f32) -> Self {
    Self {
      x: self.x * other,
      y: self.y * other,
      z: self.z * other
    }
  }
}

impl Div<f32> for Vector {
  type Output = Self;

  fn div(self, other: f32) -> Self {
    Self {
      x: self.x / other,
      y: self.y / other,
      z: self.z / other
    }
  }
}

fn calibrate_gyro(mpu: &mut Mpu6050<I2cdev>) -> Result<Calibration, Mpu6050Error<LinuxI2CError>> {
  
  let mut acc = Vector::new();
  let mut gyro = Vector::new();


  for _ in 0..CALIBRATION_SAMPLES {
    let raw_acc = mpu.get_acc_angles()?;
    let raw_gyro = mpu.get_gyro()?;

    acc.x += raw_acc.x;
    acc.y += raw_acc.y;
    // acc.z += raw_acc.z;

    gyro.x += raw_gyro.x;
    gyro.y += raw_gyro.y;
    gyro.z += raw_gyro.z;

    std::thread::sleep(Duration::from_millis(SAMPLE_INTERVAL_MS));
    
  }

  let n = CALIBRATION_SAMPLES as f32;


  let acc_bias = Vector {
    x: acc.x / n,
    y: acc.y / n,
    z: 0.0
  };

  let gyro_bias = Vector {
    x: gyro.x / n,
    y: gyro.y / n,
    z: gyro.z / n
  };


  Ok(Calibration { gyro: gyro_bias, acc: acc_bias })
  
}

fn main() -> Result<(), Mpu6050Error<LinuxI2CError>> {
  let i2c = I2cdev::new("/dev/i2c-1")
          .map_err(Mpu6050Error::I2c)?;

  let mut delay = Delay;
  let mut mpu = Mpu6050::new(i2c);

  mpu.init(&mut delay)?;

  println!("\n");

  let mut gyro = Vector::new();
  let mut acc = Vector::new();

  let calibration = calibrate_gyro(&mut mpu)?;

  // println!("done");


  loop {
    print!("\x1B[2A");

    let raw_acc = mpu.get_acc_angles()?;
    let raw_gyro = mpu.get_gyro()?;

    let mpu_acc = Vector {x: raw_acc.x, y: raw_acc.y, z: 0.0};
    let mpu_gyro = Vector {x: raw_gyro.x, y: raw_gyro.y, z: raw_gyro.z};

    // let mpu_acc = Vector {x: raw_acc.x, y: raw_acc.y, z: 0.0} - calibration.acc;
    // let mpu_gyro = Vector {x: raw_gyro.x, y: raw_gyro.y, z: raw_gyro.z} - calibration.gyro;

    gyro.smooth(&mpu_gyro);
    acc.smooth(&mpu_acc);

    println!("\rgyro: {:?}", mpu_gyro);
    println!("\racc: {:?}", mpu_acc * (180.0 / std::f32::consts::PI));

    std::thread::sleep(Duration::from_millis(100));

  }
}
