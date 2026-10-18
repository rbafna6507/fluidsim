use crate::mpu::{MPU, Vector};
use crate::imu::*;

use std::{time::{Instant}};
use i2cdev::linux::LinuxI2CError;

use std::thread::sleep;
use std::time::Duration;


const CALIBRATION_SAMPLES: i32 = 500;
const CALIBRATION_DELTA_MS: i32 = 5;
const ALPHA_FILTER: f32 = 0.95;


const FILTER_TIME_CONSTANT_SECS: f32 = 0.5;



pub fn calibrate_sensor_biases(mpu: &mut MPU) -> Result<Bias, LinuxI2CError> {
        
    let mut gyro = Vector::new();
    let mut acc = Vector::new();
    println!("Hold IMU still, flat, and facing up. Collecting calibration samples...");

    // collect CALIBRATION_SAMPLES, create an average bias while 
    for _ in 0..CALIBRATION_SAMPLES {
        let raw_gyro = mpu.get_gyro()?;
        let raw_acc = mpu.get_acc()?;

        gyro = gyro + raw_gyro;
        acc = acc + raw_acc;

        sleep(Duration::from_millis(CALIBRATION_DELTA_MS as u64));
    }

    let n = CALIBRATION_SAMPLES as f32;

    gyro = gyro / n;
    acc = acc / n;

    // expect the z value to read around 1, subtract the one to be let with the bias
    acc.z -= 1 as f32;

    println!("Done calibrating. \nGyro Bias: {:?}\n Accelerometer Bias: {:?}", gyro, acc);

    Ok(Bias {gyro: gyro, acc: acc})
}

pub fn calculate_pitch_roll(previous: Option<Orientation>, last_sample: &Sample, curr_sample: &Sample) -> Orientation {

    let accel_roll = curr_sample.acc.y.atan2(curr_sample.acc.z);
    let accel_pitch = curr_sample.acc.x.atan2((curr_sample.acc.y * curr_sample.acc.y + curr_sample.acc.z * curr_sample.acc.z).sqrt());

    // // if its the first sample, use the accelerometer to establish the starting angles.
    let Some(previous) = previous else {
        return Orientation {
            pitch: accel_pitch,
            roll: accel_roll,
        };
    };

    let dt_secs = curr_sample.sample_time.duration_since(last_sample.sample_time).as_secs_f32();

    // convert mpu driver deg/s to rad/s to match pitch/roll operations
    let predicted_roll = previous.roll + curr_sample.gyro.x.to_radians() * dt_secs;
    let predicted_pitch = previous.pitch + curr_sample.gyro.y.to_radians() * dt_secs;

    // the quicker we read from the gyro, the more we weight it's values in pitch/roll calculations
    let alpha = FILTER_TIME_CONSTANT_SECS
        / (FILTER_TIME_CONSTANT_SECS + dt_secs);

    Orientation {
        roll: alpha * predicted_roll + (1.0 - alpha) * accel_roll,
        pitch: alpha * predicted_pitch + (1.0 - alpha) * accel_pitch,
    }

}


#[derive(Debug, Clone, Copy)]
pub struct Orientation {
  pub pitch: f32,
  pub roll: f32
}
