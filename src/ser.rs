use serde_derive::{Serialize, Deserialize};
use ssmarshal::deserialize;
use crc32fast::Hasher;
use chrono::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct SolarData {
    pub battery_voltage_times_100: u16,
    pub panel_voltage_times_100: u16,
    pub panel_current_times_100: u16,
    pub load_current_times_100: u16,
    // pub bat_cell_vs: [u16; 3]
}

#[allow(dead_code)]
fn compute_checksum(data: &[u8]) -> u32 {
    let mut hasher = Hasher::new();
    hasher.update(data);
    hasher.finalize()
}

impl SolarData {
    pub fn parse(data: &[u8; SOLAR_DATA_SIZE]) -> Result<SolarData, ssmarshal::Error>  {
        // let checksum_slice = &data[SOLAR_DATA_SIZE-5..SOLAR_DATA_SIZE];
        // let sent_checksum = deserialize::<u32>(checksum_slice)?.0;
        // let actual_data = &data[0..data.len()-4];

        // if compute_checksum(actual_data) != sent_checksum {
        //     return Err(ssmarshal::Error::Custom("Checksum mismatch".to_string()));
        // }

        let actual_data = data;
        deserialize::<SolarData>(actual_data)
            .map(|(solar_data, _)| solar_data)
            .and_then(|mut solar_data| {
                if solar_data.is_valid() { 
                    // TEMP corrections
                    solar_data.battery_voltage_times_100 += 4;
                    if solar_data.panel_current_times_100 < 5 {
                        solar_data.panel_current_times_100 = 0;
                    }
                    Ok(solar_data) 
                } else { 
                    Err(ssmarshal::Error::Custom("watts outside range".to_string()))
                }
            })
    }


    pub fn panel_watts(&self) -> f32 { 
        self.panel_v() * self.panel_current()
    }

    pub fn load_watts(&self) -> f32 {
        self.bat_v() * self.load_current()
    }

    pub fn bat_v(&self) -> f32 {
        const WIRE_RESISTANCE: f32 = 0.00866;
        let v = self.battery_voltage_times_100 as f32 / 100.0;
        
        // let net_current = self.panel_current() - self.load_current();

        // if net_current > 0.0 {
            // v - net_current * WIRE_RESISTANCE
        // } else {
            v
        // }
    }

    pub fn panel_v(&self) -> f32 {
        self.panel_voltage_times_100 as f32 / 100.0
    }

    pub fn load_current(&self) -> f32 {
        self.load_current_times_100 as f32 / 100.0
    }

    pub fn panel_current(&self) -> f32 {
        self.panel_current_times_100 as f32 / 100.0
    }

    pub fn is_valid(&self) -> bool {
        self.panel_watts() < 400.0 && self.bat_v() > 2.8*4.0 && self.load_watts() < 400.0
    }

    // pub fn cell_v(&self, index: usize) -> f32 {
    //     const VREF: f32 = 3.30;
    //     const CELL_DIVIDERS: [f32; 3] = [2.001, 3.2085, 5.6624];
    //     let mut raw_v = if index < 3 {
    //         self.bat_cell_vs[index] as f32 / 4096.0 * CELL_DIVIDERS[index] * VREF
    //     } else {
    //         self.bat_v()
    //     };

    //     // subtract the battery voltages below this one
    //     for i in 0..index {
    //         raw_v -= self.cell_v(i);
    //     }
    //     raw_v
    // }

    // pub fn cell_v_raw(&self, index: usize) -> f32 {
    //     const VREF: f32 = 3.30;
    //      self.bat_cell_vs[index] as f32 / 4096.0 * VREF
    // }
}

pub struct CumulativeSolarData {
    pub snapshot: SolarData,
    pub time: DateTime<Utc>,
    interval_s: f32,
}

impl CumulativeSolarData {
    pub fn from_snapshot(prev_datetime: DateTime<Utc>, solar_data: SolarData) -> CumulativeSolarData {
        let cur = Utc::now();
        let diff_s = (cur.timestamp_nanos() - prev_datetime.timestamp_nanos()) as f32 / 1_000_000_000.0;

        CumulativeSolarData { snapshot: solar_data, interval_s: diff_s, time: cur }
    }

    pub fn panel_watt_s(&self) -> f32 {
        self.snapshot.panel_watts() * self.interval_s
    }

    pub fn panel_watt_h(&self) -> f32 {
        self.panel_watt_s() / 60.0 / 60.0
    }

    pub fn load_watt_s(&self) -> f32 {
        self.snapshot.load_watts() * self.interval_s
    }

    pub fn load_watt_h(&self) -> f32 {
        self.load_watt_s() / 60.0 / 60.0
    }
}

 
pub const SOLAR_DATA_SIZE: usize = core::mem::size_of::<SolarData>() + 4;


