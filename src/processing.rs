use crate::mpu::{self, Vector};
use std::{any::Any, thread, time::Duration};


const CALIBRATION_SAMPLES: i32 = 500;
const CALIBRATION_DELTA_MS: i32 = 5;

pub struct Bias {
    pub gyro: Vector,
    pub acc: Vector
}


#[derive(Default)]
pub struct Processor {
    pub mpu: mpu::MPU,

}

impl Processor {
    pub fn new(mpu: &mut MPU) -> Processor {
        Processor { mpu: MPU }
    }

    pub fn calibrate_sensor_biases(self) -> Bias {
        
        let gyro = Vector::new();
        let acc = Vector::new();
        println!("Hold IMU still, flat, and facing up. Collecting calibration samples...");

        // collect CALIBRATION_SAMPLES, create an average bias while 
        for _ in 0..CALIBRATION_SAMPLES {
            let raw_gyro =self.mpu.get_gyro()?;
            let raw_acc = self.mpu.get_acc()?;

            gryo += raw_gyro;
            acc += raw_acc;

            std::thread::sleep(Duration::from_millis(SAMPLE_INTERVAL_MS));
        }

        let n = CALIBRATION_SAMPLES as f32;

        gyro = gyro / n;
        acc = acc / n;

        // expect the z value to read around 1, subtract the one to be let with the bias
        acc.z -= 1;

        println!("Done calibrating. \nGyro Bias: {:?}\n Accelerometer Bias: {:?}", gyro, acc);

        Bias {gyro: gyro, acc: acc}
    }

    pub fn calculate_pitch_roll(gyro: Vector, acc: Vector) {


        unimplemented!()
    }



}
