use crate::mpu::{self, MPU, Vector};
use std::{any::Any, thread, time::Duration};
use i2cdev::linux::LinuxI2CError;


const CALIBRATION_SAMPLES: i32 = 500;
const CALIBRATION_DELTA_MS: i32 = 5;

pub struct Bias {
    pub gyro: Vector,
    pub acc: Vector
}


#[derive(Default)]
pub struct Processor;

impl Processor {

    pub fn calibrate_sensor_biases(self, mpu: &mut MPU) -> Result<Bias, LinuxI2CError> {
        
        let mut gyro = Vector::new();
        let mut acc = Vector::new();
        println!("Hold IMU still, flat, and facing up. Collecting calibration samples...");

        // collect CALIBRATION_SAMPLES, create an average bias while 
        for _ in 0..CALIBRATION_SAMPLES {
            let raw_gyro = mpu.get_gyro()?;
            let raw_acc = mpu.get_acc()?;

            gyro = gyro + raw_gyro;
            acc = acc + raw_acc;

            std::thread::sleep(Duration::from_millis(CALIBRATION_DELTA_MS as u64));
        }

        let n = CALIBRATION_SAMPLES as f32;

        gyro = gyro / n;
        acc = acc / n;

        // expect the z value to read around 1, subtract the one to be let with the bias
        acc.z -= 1 as f32;

        println!("Done calibrating. \nGyro Bias: {:?}\n Accelerometer Bias: {:?}", gyro, acc);

        Ok(Bias {gyro: gyro, acc: acc})
    }

    pub fn calculate_pitch_roll(gyro: Vector, acc: Vector) {


        unimplemented!()
    }



}
