use linux_embedded_hal::*;
use embedded_hal::blocking::i2c::{Write, WriteRead};
use i2cdev::linux::LinuxI2CError;

use std::ops::Sub;
use std::ops::Mul;
use std::ops::Div;
use std::ops::Add;

use crate::MPU_ADDR;


const WHO_AM_I: u8 = 0x75;
const PWR_MGMT_1: u8 = 0x6B;
const ACCEL_XOUT_H: u8 = 0x3B;
const GYRO_XOUT_H: u8 = 0x43;

const ACCEL_SCALE: f32 = 16384.0; // ±2g
const GYRO_SCALE: f32 = 131.0;    // ±250 deg/s

const ALPHA: f32 = 0.15;

pub struct Measurements {
    pub gyro: Vector,
    pub acc: Vector
}



pub struct MPU {
    i2c: I2cdev,
    address: u8,
}


impl MPU {

    pub fn new(i2c_: I2cdev) -> MPU {

        println!("new mpu declared");

        MPU {
            i2c: i2c_,
            address: MPU_ADDR
        }
    }

    pub fn new_with_address(i2c_: I2cdev, address_: u8) -> MPU {

        MPU {
            i2c: i2c_,
            address: address_
        }
    }

    pub fn init(&mut self) -> Result<(), LinuxI2CError> {

        println!("initializing...");

        let id = self.read_register(WHO_AM_I)?;
        println!("WHO_AM_I: {id:#04x}");
        
        if id != 0x68 {
            println!("id is not 0x68");
            panic!("Unexpected MPU6050 device ID");
        }

        // Wake sensor and select internal oscillator.
        self.write_register(PWR_MGMT_1, 0x00)?;

        println!("sensor awake");

        // Accelerometer ±2g
        self.write_register(0x1C, 0x00)?;

        println!("accelerometer written");

        // Gyroscope ±250 degrees/second
        self.write_register(0x1B, 0x00)?;

        println!("gyro written");

        println!("initializing complete");

        Ok(())
    }

    pub fn read_registers(&mut self, reg: u8, length: usize) -> Result<Vec<u8>, LinuxI2CError> {
        let mut buffer = vec![0u8; length];

        self.i2c.write_read(
            self.address,
            &[reg],
            &mut buffer
        )?;

        Ok(buffer)
    }


    pub fn read_register(&mut self, reg: u8) -> Result<u8, LinuxI2CError> {
        let mut buffer = [0u8; 1];

        self.i2c.write_read(
            self.address,
            &[reg],
            &mut buffer,
        )?;

        Ok(buffer[0])
    }

    pub fn write_register(&mut self, reg: u8, value: u8) -> Result<(), LinuxI2CError> {
        self.i2c.write(
            self.address,
            &[reg, value],
        )
    }

    pub fn get_acc(&mut self) -> Result<Vector, LinuxI2CError> {

        let acc_values = self.read_registers(ACCEL_XOUT_H, 6)?;

        let read_i16_as_f32 = |i: usize| {
            i16::from_be_bytes([acc_values[i], acc_values[i+1]]) as f32
        };
    
        // divide by ACCEL_SCALE to convert measurment into g
        let acc_x = read_i16_as_f32(0) / ACCEL_SCALE;
        let acc_y = read_i16_as_f32(2) / ACCEL_SCALE;
        let acc_z = read_i16_as_f32(4) / ACCEL_SCALE;

        let acc:Vector = Vector {x: acc_x, y: acc_y, z: acc_z};

        Ok(acc)
    }


    pub fn get_gyro(&mut self) -> Result<Vector, LinuxI2CError> {
        let gyro_values = self.read_registers(GYRO_XOUT_H, 6)?;

        let read_i16_as_f32 = |i: usize| {
            i16::from_be_bytes([gyro_values[i], gyro_values[i+1]]) as f32
        };
        
        // divide by GYRO_SCALE to convert measurement in deg/s
        let gyro_values_x = read_i16_as_f32(0) / GYRO_SCALE;
        let gyro_values_y = read_i16_as_f32(2) / GYRO_SCALE;
        let gyro_values_z = read_i16_as_f32(4) / GYRO_SCALE;

        let gyro_values: Vector = Vector {x: gyro_values_x, y: gyro_values_y, z: gyro_values_z};

        Ok(gyro_values)
    }

    pub fn read(&mut self) -> Result<Measurements, LinuxI2CError> {
        let buf = self.read_registers(ACCEL_XOUT_H, 14)?;

        let read_i16_as_f32 = |i: usize| {
            i16::from_be_bytes([buf[i], buf[i+1]]) as f32
        };

        let ax = read_i16_as_f32(0) / ACCEL_SCALE;
        let ay = read_i16_as_f32(2) / ACCEL_SCALE;
        let az = read_i16_as_f32(4) / ACCEL_SCALE;

        // skip temperature reading at registers 6 and 7

        let gx = read_i16_as_f32(8) / GYRO_SCALE;
        let gy = read_i16_as_f32(10) / GYRO_SCALE;
        let gz = read_i16_as_f32(12) / GYRO_SCALE;

        // acc vals in g
        let acc_vals = Vector {x: ax, y: ay, z: az};

        // gyro vals in deg/s
        let gyro_vals = Vector {x: gx, y: gy, z: gz};

        Ok(Measurements { gyro: gyro_vals, acc: acc_vals })

    }


}


#[derive(Default, Debug, Clone, Copy)]
 pub struct Vector {
  pub x: f32,
  pub y: f32,
  pub z: f32
}

impl Vector {

  pub fn new() -> Vector {
    Vector { x: 0.0, y: 0.0, z: 0.0 }
  }

  pub fn smooth(&mut self, new: & Vector) {

    self.x = self.x + ALPHA * (new.x - self.x);
    self.y = self.y + ALPHA * (new.y - self.y);
    self.z = self.z + ALPHA * (new.z - self.z)

  }

  pub fn magnitude(&self) -> f32 {
    (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
  }

}

impl Add for Vector {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z
        }
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
