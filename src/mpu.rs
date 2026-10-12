use linux_embedded_hal::*;
use i2cdev::linux::*;
use i2cdev::linux::LinuxI2CError



const MPU_ADDR: u8 = 0x68;

const WHO_AM_I: u8 = 0x75;
const PWR_MGMT_1: u8 = 0x6B;
const ACCEL_XOUT_H: u8 = 0x3B;

const ACCEL_SCALE: f32 = 16384.0; // ±2g
const GYRO_SCALE: f32 = 131.0;    // ±250 deg/s


struct MPU {
    i2c: I2cdev,
    address: u8,
}



impl MPU {

    fn new(i2c: I2cdev, address: u8) -> MPU {

        MPU {
            i2c: i2c,
            address: address
        }
    }

    fn init(self) -> Result<(), LinuxI2CError> {

        let id = self.read_register(WHO_AM_I);
        
        if id != 0x68 {
            panic!("Unexpected MPU6050 device ID: {id:#04x}");
        }

        // Wake sensor and select internal oscillator.
        self.write_register(PWR_MGMT_1, 0x00)?;

        // Accelerometer ±2g
        self.write_register(0x1C, 0x00)?;

        // Gyroscope ±250 degrees/second
        self.write_register(0x1B, 0x00)?;

        Ok(())
    }

    fn read_register(self, reg: u8) -> Result<i16, LinuxI2CError> {
        unimplemented!()
    }

    fn write_register(self, reg: u8, value: u8) -> Result<i16, LinuxI2CError> {
        unimplemented!()
    }

    fn get_acc() {
        unimplemented!()
    }

    fn get_gyro() {
        unimplemented!()
    }


}