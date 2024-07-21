// extern crate chrono;
// // extern crate i2cdev;
// extern crate ssmarshal;
// extern crate crc32fast;
// extern crate rumqtt;
// extern crate energy_monitor;
// extern crate tokio_serial;

// use i2cdev::linux::LinuxI2CDevice;
// use i2cdev::core::I2CDevice;

use std::{thread, env};
use chrono::prelude::*;
use std::fs;
use rumqtt::MqttOptions;
use energy_monitor::ser::{SolarData, CumulativeSolarData, SOLAR_DATA_SIZE};
use energy_monitor::data::Publisher;
use tokio_serial::{Serial, SerialPortSettings};

// #[cfg(all(feature = "rtu", feature = "sync"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    use tokio_modbus::prelude::*;

    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("Usage: energy-monitor [influxdbhost:port] [mqtthost:port]");
        return Ok(());
    }

    // let mut device = LinuxI2CDevice::new(
    //         "/dev/i2c-1",
    //         0x21).unwrap();
    

    let slave = Slave(0x1);
    let mut settings = SerialPortSettings::default();
    settings.baud_rate = 115200;
    let port = Serial::from_path("/dev/ttyUSB0", &settings).unwrap();

    let mut ctx = rtu::connect_slave(port, slave).await?;

    let mqtt_params: Vec<&str> = (&args[2]).split(":").collect();

    let mqtt_options = MqttOptions::new("energy-monitor", mqtt_params[0], mqtt_params[1].parse::<u16>().unwrap());
    let influx = format!("http://{}", args[1]);
    let mut publisher = Publisher::new(mqtt_options, &influx);

    let mut now = Utc::now();
    let mut cum_watt_h = fs::read_to_string("last_wh").map(|r| r.trim().parse().unwrap_or(0.0)).unwrap_or(0.0);
    let mut house_cum_watt_h = fs::read_to_string("house_last_wh").map(|r| r.trim().parse().unwrap_or(0.0)).unwrap_or(0.0);

    loop {
        // let mut temp = [0_u8; 12];
        let panel_voltage = ctx.read_input_registers(0x3100, 1).await;
        let panel_current = ctx.read_input_registers(0x3101, 1).await;
        let battery_voltage = ctx.read_input_registers(0x3104, 1).await;
        // let battery_current = ctx.read_input_registers(0x3105, 1).await;
        let load_current = ctx.read_input_registers(0x310D, 1).await;
        let battery_percent = ctx.read_input_registers(0x311A, 1).await;

        if panel_voltage.is_ok() && panel_current.is_ok() &&
            battery_voltage.is_ok() && 
            load_current.is_ok() && battery_percent.is_ok() {

            let solar_data = SolarData {
                battery_voltage_times_100: battery_voltage.unwrap()[0],
                panel_voltage_times_100: panel_voltage.unwrap()[0],
                panel_current_times_100: panel_current.unwrap()[0],
                load_current_times_100: load_current.unwrap()[0]
            };

            let cum_solar_data = CumulativeSolarData::from_snapshot(now, solar_data);


            // let result = SolarData { battery_voltage_times_100: result
            //     .map(|solar_data| CumulativeSolarData::from_snapshot(now, solar_data));

            // match result {
            //     Ok(cum_solar_data) => {
                    if cum_solar_data.time.with_timezone(&Local).day() != now.with_timezone(&Local).day() {
                        cum_watt_h = 0.0;
                        house_cum_watt_h = 0.0;
                    }

                    cum_watt_h += cum_solar_data.panel_watt_h();
                    house_cum_watt_h += cum_solar_data.load_watt_h();
                    now = cum_solar_data.time;

                    publisher.publish(&cum_solar_data, cum_watt_h, house_cum_watt_h).await;

                    println!("{}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}", 
                             Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
                             cum_solar_data.snapshot.panel_v(), 
                             cum_solar_data.snapshot.bat_v(), 
                             cum_solar_data.snapshot.panel_current(),
                             cum_solar_data.snapshot.panel_watts(),
                             cum_solar_data.panel_watt_s(),
                             cum_watt_h,
                             cum_solar_data.snapshot.load_current(),
                             cum_solar_data.snapshot.load_watts(),
                             cum_solar_data.load_watt_s(),
                             house_cum_watt_h,
                             // cum_solar_data.snapshot.cell_v(0),
                             // cum_solar_data.snapshot.cell_v(1), 
                             // cum_solar_data.snapshot.cell_v(2),
                             // cum_solar_data.snapshot.bat_cell_vs[0],
                             // cum_solar_data.snapshot.bat_cell_vs[1],
                             // cum_solar_data.snapshot.bat_cell_vs[2],
                             // cum_solar_data.snapshot.cell_v_raw(0),
                             // cum_solar_data.snapshot.cell_v_raw(1), 
                             // cum_solar_data.snapshot.cell_v_raw(2),
                             );
                    
                    if let Err(_) = fs::write("last_wh", &cum_watt_h.to_string()) { }
                    if let Err(_) = fs::write("house_last_wh", &house_cum_watt_h.to_string()) { }
            //     } 
            //     Err(x) => eprintln!("Error parsing: {}", x)
            // }

        } else {
            eprintln!("Could not read values");
        }

        thread::sleep(std::time::Duration::from_millis(800_u64));
    }
}
