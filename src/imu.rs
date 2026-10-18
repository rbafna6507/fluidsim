use crate::mpu;
use crate::processing;

use mpu::*;
use processing::*;
use std::{time::{Duration, Instant}};
use i2cdev::linux::LinuxI2CError;

#[derive(Clone, Copy, Debug)]
pub struct Bias {
    pub gyro: Vector,
    pub acc: Vector
}

#[derive(Clone, Copy, Debug)]
pub struct Sample {
    pub sample_time: Instant,
    pub gyro: Vector, // bias-corrected degrees/second
    pub acc: Vector,  // bias-corrected g
}


pub struct IMU {
    pub mpu: MPU,

    pub last_sample: Sample,
    pub orientation: Option<Orientation>,
    pub biases: Bias


}

impl IMU {
    pub fn new(mut mpu: MPU) -> Result<Self, LinuxI2CError> {

        mpu.init()?;

        let bias = processing::calibrate_sensor_biases(&mut mpu)?;
        let measurements = mpu.read()?;

        let sample: Sample = Sample {sample_time: Instant::now(), gyro: measurements.gyro, acc: measurements.acc};
        // let orientation = processor.calculate_pitch_roll(dt, gyro, acc)
        

        Ok(Self {mpu: mpu, last_sample: sample, orientation: None, biases: bias})
    }

    pub fn update(&mut self) -> Result<(), LinuxI2CError>{

        let raw_gyro = self.mpu.get_gyro()? - self.biases.gyro;
        let raw_acc = self.mpu.get_acc()? - self.biases.acc;
        let now = Instant::now();

        let curr_sample = Sample { sample_time: now, gyro: raw_gyro, acc: raw_acc };

        let orientation = processing::calculate_pitch_roll(self.orientation, &self.last_sample, &curr_sample);

        self.last_sample = curr_sample;
        self.orientation = Some(orientation);
        
        Ok(())
    }
}