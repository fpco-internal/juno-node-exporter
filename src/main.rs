use anyhow::*;
use chrono::offset::Utc;
use chrono::*;
use prometheus_exporter::prometheus::register_gauge;
use reqwest::blocking::get;
use std::env::var;
use std::net::SocketAddr;
use std::time::Duration;

fn main() -> Result<()> {
    let port_for_exporter_to_listen = var("JE_PORT")
        .map_err(|x| anyhow!("{:?}", x))
        .and_then(|x| x.parse().map_err(|x| anyhow!("{:?}", x)))
        .unwrap_or(9100);
    let target = var("TARGET").unwrap_or_else(|_| "http://localhost:26657/status".to_string());
    let gauge_name = var("GAUGE_NAME")?;
    let gauge_help = var("GAUGE_HELP")?;

    let exporter_listen_address: SocketAddr =
        format!("0.0.0.0:{port_for_exporter_to_listen}").parse()?;
    let exporter = prometheus_exporter::start(exporter_listen_address)?;

    let duration = Duration::from_secs(3600); // Checking once an hour should be enough

    let metric = register_gauge!(gauge_name, gauge_help)?;

    loop {
        let body =
            &json::parse(&(get(&target)?.text()?))?["result"]["sync_info"]["latest_block_time"];
        let datetime_string = body
            .as_str()
            .ok_or_else(|| anyhow!("Failed to extract latest_block_time"))?;
        let datetime = datetime_string.parse::<DateTime<Utc>>()?;
        let now = Utc::now();
        let span = now - datetime;
        metric.set(span.num_hours() as f64);
        let _guard = exporter.wait_duration(duration);
    }
}
