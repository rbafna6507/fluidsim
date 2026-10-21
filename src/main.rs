mod imu;
mod mpu;
mod processing;
mod telemetry;
mod sim;

use std::{
    env,
    error::Error,
    net::{SocketAddr, UdpSocket},
    time::{Duration, Instant},
};

use imu::*;
use mpu::MPU;
use processing::*;

use linux_embedded_hal::I2cdev;

use crate::mpu::Vector;

const MPU_ADDR: u8 = 0x68;
const PRINT_PERIOD_MS: u128 = 100;
const ORIENTATION_ALPHA: f32 = 0.15;

struct State {
    imu: IMU,

    display_orientation: Orientation,
    display_gyro: Vector,
    display_acc: Vector,
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("running");

    let udp_target = udp_target_from_args()?;
    let udp_socket = udp_target
        .map(|_| UdpSocket::bind("0.0.0.0:0"))
        .transpose()?;
    if let Some(target) = udp_target {
        println!("Streaming pitch/roll to {target}");
    }

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

        state.display_gyro.smooth(&state.imu.last_sample.gyro);
        state.display_acc.smooth(&state.imu.last_sample.acc);
        state.display_orientation.pitch += ORIENTATION_ALPHA
            * (state.imu.orientation.unwrap().pitch - state.display_orientation.pitch);
        state.display_orientation.roll += ORIENTATION_ALPHA
            * (state.imu.orientation.unwrap().roll - state.display_orientation.roll);


        if let (Some(socket), Some(target)) = (&udp_socket, udp_target) {
            let packet = telemetry::encode_orientation(state.display_orientation);
            socket.send_to(packet.as_bytes(), target)?;
        }

        if dt >= PRINT_PERIOD_MS {
            print!("\x1B[4A");

            println!("\rgyro: {:?}", state.display_gyro);
            println!("\racc: {:?}", state.display_acc);
            println!("\rpitch: {:?}", state.display_orientation.pitch);
            println!("\rroll: {:?}", state.display_orientation.roll);
            
            last_time = Instant::now();
        }

    }
}

fn udp_target_from_args() -> Result<Option<SocketAddr>, Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let Some(option) = args.next() else {
        return Ok(None);
    };

    if option != "--udp" {
        return Err(format!("unknown argument: {option}; usage: fluid [--udp IP:PORT]").into());
    }

    let address = args
        .next()
        .ok_or("missing address; usage: fluid --udp IP:PORT")?;
    let target = address.parse::<SocketAddr>()?;

    if args.next().is_some() {
        return Err("too many arguments; usage: fluid [--udp IP:PORT]".into());
    }

    Ok(Some(target))
}
