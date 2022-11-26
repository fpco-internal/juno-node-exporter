use anyhow::*;
use chrono::offset::Utc;
use chrono::*;
use prometheus_exporter::prometheus::register_int_gauge_vec;
use reqwest::blocking::get;
use std::env::var;
use std::net::SocketAddr;
use std::time::Duration;

fn main() -> Result<()> {
    eprintln!(env!("CARGO_PKG_VERSION"));
    let port_for_exporter_to_listen = var("ENDPOINT_PORT")
        .map_err(|x| anyhow!("{:?}", x))
        .and_then(|x| x.parse().map_err(|x| anyhow!("{:?}", x)))
        .unwrap_or(9100);
    let gauge_name = var("GAUGE_NAME")?;
    let gauge_help = var("GAUGE_HELP")?;
    let interval = var("INTERVAL")
        .map_err(|e| anyhow!("{:?}", e))
        .and_then(|x| x.parse::<u64>().map_err(|e| anyhow!("{:?}", e)))
        .unwrap_or(3600);
    let targets = var("TARGETS").unwrap_or_else(|_| "http://localhost:26657/status".to_string());
    let targets: Vec<_> = targets.split(',').collect();
    let target_names = var("TARGET_NAMES").unwrap_or_else(|_| "localhost".to_string());
    let target_names: Vec<_> = target_names.split(',').collect();
    assert!(targets.len() == target_names.len());

    let exporter_listen_address: SocketAddr =
        format!("0.0.0.0:{port_for_exporter_to_listen}").parse()?;
    let exporter = prometheus_exporter::start(exporter_listen_address)?;

    let duration = Duration::from_secs(interval);

    let metric = register_int_gauge_vec!(gauge_name, gauge_help, &target_names)?;

    loop {
        for target in targets.iter() {
            let body =
                &json::parse(&(get(*target)?.text()?))?["result"]["sync_info"]["latest_block_time"];
            let datetime_string = body
                .as_str()
                .ok_or_else(|| anyhow!("Failed to extract latest_block_time"))?;
            let datetime = datetime_string.parse::<DateTime<Utc>>()?;
            let now = Utc::now();
            let span = now - datetime;
            metric
                .get_metric_with_label_values(&[target])?
                .set(span.num_hours());
        }
        let _guard = exporter.wait_duration(duration);
    }
}
