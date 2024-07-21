use crate::ser::CumulativeSolarData;
use tokio::runtime::Runtime;
use influxdb::{Client, Query, Timestamp};
use influxdb::InfluxDbWriteable;
use chrono::{DateTime, Utc};

use rumqtt::{MqttClient, MqttOptions, QoS};

pub struct Publisher {
    mqtt_client: MqttClient,
    influx_client: Client,
    runtime: Runtime,
}

fn publish_message<T>(mqtt_client: &mut MqttClient, topic: &str, message: T) 
    where T : ToString {
    if let Err(e) = mqtt_client.publish(topic, QoS::AtMostOnce, message.to_string()) {
        eprintln!("Error publishing message: {}", e); 
    } 
}

#[derive(InfluxDbWriteable)]
pub struct SolarReading {
    time: DateTime<Utc>,
    panel_watts: f32,
    panel_kwh: f32,
    panel_watt_s: f32,
    load_watts: f32,
    load_kwh: f32,
    load_watt_s: f32,
    batt_v: f32,
    batt_percent: f32,
}

impl Publisher {
    pub fn new(options: MqttOptions, influx_host: &str) -> Self {
        let (mqtt_client, _notifications) = MqttClient::start(options)
            .expect("Could not connect to mqtt. Is it running?");
        let influx_client = Client::new(influx_host, "energy");
        Self { 
            mqtt_client,
            influx_client: influx_client,
            runtime: Runtime::new().expect("Could not create tokio runtime"),
        }
    }

    pub async fn publish(&mut self, cum_solar_data: &CumulativeSolarData, cum_watt_h: f32, house_cum_watt_h: f32) {
        let batt_percent = crate::v_to_percent(cum_solar_data.snapshot.bat_v() / 4.0).unwrap_or(0.0);
        let snap = &cum_solar_data.snapshot;
        publish_message(&mut self.mqtt_client, "solar/watt", snap.panel_watts());
        publish_message(&mut self.mqtt_client, "solar/kwh", cum_watt_h / 1000.0);
        publish_message(&mut self.mqtt_client, "house/watt", snap.load_watts());
        publish_message(&mut self.mqtt_client, "house/kwh", house_cum_watt_h / 1000.0);
        publish_message(&mut self.mqtt_client, "powerwall/percent", 
                        format!("{},{}",  
                                batt_percent,
                                snap.bat_v()));
        // publish_message(&mut self.mqtt_client, "bms/voltages", 
        //                 format!("{},{},{},{}", 
        //                         snap.cell_v(0),
        //                         snap.cell_v(1),
        //                         snap.cell_v(2),
        //                         snap.cell_v(3)));

        let solar_reading = SolarReading {
            time: cum_solar_data.time,
            panel_watts: cum_solar_data.snapshot.panel_watts(),
            panel_kwh: cum_watt_h  / 1000.0,
            panel_watt_s: cum_solar_data.panel_watt_s(),
            load_watts: cum_solar_data.snapshot.load_watts(),
            load_kwh: house_cum_watt_h / 1000.0,
            load_watt_s: cum_solar_data.load_watt_s(),
            batt_v: cum_solar_data.snapshot.bat_v(),
            batt_percent: batt_percent
        };

        if let Err(e) = self.influx_client.query(&solar_reading.into_query("energy")).await {
            eprintln!("Error: {}", e);
        }
    }
}

