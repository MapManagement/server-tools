use config::{Config, ConfigError};
use reqwest::header::AUTHORIZATION;
use serde::Deserialize;
use serde_json::Value;
use std::error::Error;
use std::fmt;
use std::process::{Command, Stdio};
use std::time::SystemTime;

#[derive(Debug, Deserialize)]
struct BorgFluxConfig {
    influx_url: String,
    influx_token: String,
    influx_org: String,
    influx_bucket: String,
    hostname: String,
    borg_repository: String,
    borg_source_path: String,
}

impl BorgFluxConfig {
    pub fn new(file_path: &str) -> Result<Self, ConfigError> {
        let config = Config::builder()
            .add_source(config::File::with_name(file_path))
            .build()?;

        config.try_deserialize()
    }
}

struct InfluxTag {
    name: String,
    value: String,
}

struct InfluxField {
    name: String,
    value: InfluxFieldValue,
}

enum InfluxFieldValue {
    Float(f64),
    Int(i64),
    String(String),
}

struct InfluxPoint {
    measurement: String,
    tags: Vec<InfluxTag>,
    fields: Vec<InfluxField>,
}

impl fmt::Display for InfluxFieldValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Float(v) => write!(f, "{}", v),
            Self::Int(v) => write!(f, "{}", v),
            Self::String(v) => write!(f, "{}", v),
        }
    }
}

struct Backup {
    host: String,
    backup_name: String,
    encryption: String,
    repo_location: String,
    duration: f64,
    compressed_size: i64,
    deduplicated_size: i64,
    number_of_files: i64,
    original_size: i64,
}

pub fn run_borgflux(file_path: &str) {
    let env_output = read_config_file(file_path);

    if env_output.is_err() {
        eprintln!("Error: {}", env_output.err().unwrap());
        return;
    }

    let credentials = env_output.unwrap();

    send_frame_point(&credentials, "backup_start".to_string());

    let json_output = run_borg_backup(&credentials.borg_repository, &credentials.borg_source_path);

    if json_output.is_err() {
        send_error_point(&credentials, &json_output.to_owned().unwrap_err());
        return;
    }

    let json_value = read_borg_json_output(json_output.to_owned().unwrap());

    if json_value.is_err() {
        send_error_point(&credentials, "Couldn't read the JSON file correctly!");
        return;
    }

    let backup_stats = extract_data_from_json(json_value.unwrap(), credentials.hostname.to_owned());
    let influx_backup_point = create_influx_point_from_backup(backup_stats);

    write_to_influx(influx_backup_point, &credentials);

    send_frame_point(&credentials, "backup_end".to_string());
}

fn read_config_file(file_path: &str) -> Result<BorgFluxConfig, String> {
    let result = BorgFluxConfig::new(file_path);

    match result {
        Ok(config) => Ok(config),
        Err(_) => Err("Couldn't read the configuration file!".to_string()),
    }
}

fn read_borg_json_output(json_output: String) -> Result<Value, Box<dyn Error>> {
    let json_value: Value = serde_json::from_str(&json_output)?;

    return Ok(json_value);
}

fn extract_data_from_json(json_data: Value, host: String) -> Backup {
    Backup {
        host,
        backup_name: json_data["archive"]["name"].to_string(),
        encryption: json_data["encryption"]["mode"].to_string(),
        repo_location: json_data["repository"]["location"].to_string(),
        duration: json_data["archive"]["duration"].as_f64().unwrap(),
        compressed_size: json_data["archive"]["stats"]["compressed_size"]
            .as_i64()
            .unwrap(),
        deduplicated_size: json_data["archive"]["stats"]["deduplicated_size"]
            .as_i64()
            .unwrap(),
        number_of_files: json_data["archive"]["stats"]["nfiles"].as_i64().unwrap(),
        original_size: json_data["archive"]["stats"]["original_size"]
            .as_i64()
            .unwrap(),
    }
}

fn create_influx_point_from_backup(backup: Backup) -> InfluxPoint {
    let tags: Vec<InfluxTag> = vec![
        InfluxTag {
            name: "host".to_string(),
            value: backup.host,
        },
        InfluxTag {
            name: "backup_name".to_string(),
            value: backup.backup_name,
        },
        InfluxTag {
            name: "encryption".to_string(),
            value: backup.encryption,
        },
        InfluxTag {
            name: "repo_location".to_string(),
            value: backup.repo_location,
        },
    ];
    let fields: Vec<InfluxField> = vec![
        InfluxField {
            name: "duration".to_string(),
            value: InfluxFieldValue::Float(backup.duration),
        },
        InfluxField {
            name: "compressed_size".to_string(),
            value: InfluxFieldValue::Int(backup.compressed_size),
        },
        InfluxField {
            name: "deduplicated_size".to_string(),
            value: InfluxFieldValue::Int(backup.deduplicated_size),
        },
        InfluxField {
            name: "number_of_files".to_string(),
            value: InfluxFieldValue::Int(backup.number_of_files),
        },
        InfluxField {
            name: "original_size".to_string(),
            value: InfluxFieldValue::Int(backup.original_size),
        },
    ];

    InfluxPoint {
        measurement: "backup_data".to_string(),
        tags,
        fields,
    }
}

fn send_frame_point(credentials: &BorgFluxConfig, measurement: String) {
    let tags: Vec<InfluxTag> = vec![
        InfluxTag {
            name: "host".to_string(),
            value: credentials.hostname.to_owned(),
        },
        InfluxTag {
            name: "repository".to_string(),
            value: credentials.borg_repository.to_owned(),
        },
        InfluxTag {
            name: "source_path".to_string(),
            value: credentials.borg_source_path.to_owned(),
        },
    ];

    let fields: Vec<InfluxField> = vec![InfluxField {
        name: "dummy".to_string(),
        value: InfluxFieldValue::Int(0),
    }];

    let point = InfluxPoint {
        measurement,
        tags,
        fields,
    };

    write_to_influx(point, credentials);
}

fn send_error_point(credentials: &BorgFluxConfig, error_text: &str) {
    let tags: Vec<InfluxTag> = vec![
        InfluxTag {
            name: "host".to_string(),
            value: credentials.hostname.to_owned(),
        },
        InfluxTag {
            name: "repository".to_string(),
            value: credentials.borg_repository.to_owned(),
        },
        InfluxTag {
            name: "source_path".to_string(),
            value: credentials.borg_source_path.to_owned(),
        },
    ];

    let fields: Vec<InfluxField> = vec![InfluxField {
        name: "error".to_string(),
        value: InfluxFieldValue::String(error_text.to_string()),
    }];

    let point = InfluxPoint {
        measurement: "backup_error".to_string(),
        tags,
        fields,
    };

    eprintln!("Error: {}", error_text);
    write_to_influx(point, credentials);
}

fn build_raw_data_from_point(point: InfluxPoint) -> String {
    let mut raw_data = point.measurement;

    for tag in point.tags {
        raw_data = format!("{},{}={}", raw_data, tag.name, tag.value);
    }

    if point.fields.len() != 0 {
        raw_data = format!("{} ", raw_data);
    }

    let mut counter = 1;
    let number_of_fields = point.fields.len().to_owned();

    for field in point.fields {
        raw_data = format!("{}{}={}", raw_data, field.name, field.value);

        if counter < number_of_fields {
            raw_data = format!("{},", raw_data);
        }

        counter += 1;
    }

    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    format!("{} {}", raw_data, timestamp)
}

fn write_to_influx(point: InfluxPoint, credentials: &BorgFluxConfig) {
    let client = reqwest::blocking::Client::new();
    let body = build_raw_data_from_point(point);
    let api_url = format!(
        "{}/api/v2/write?bucket={}&org={}&precision=s",
        credentials.influx_url.to_owned(),
        credentials.influx_bucket.to_owned(),
        credentials.influx_org.to_owned()
    );

    let result = client
        .post(api_url)
        .body(body)
        .header(
            AUTHORIZATION,
            "Token ".to_owned() + &credentials.influx_token.to_owned(),
        )
        .send();

    if result.is_err() {
        send_error_point(&credentials, "Unable to send data to InfluxDB!");
    }
}

fn run_borg_backup(repository: &String, source_path: &String) -> Result<String, String> {
    let is_borg_installed = Command::new("which")
        .stdout(Stdio::null())
        .arg("borg")
        .status();

    if is_borg_installed.is_err() || !is_borg_installed.unwrap().success() {
        return Err("BorgBackup is not installed!".to_string());
    }

    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string();

    let backup_repo = format!("{}::{}", repository.to_owned(), timestamp);

    let output = Command::new("borg")
        .arg("create")
        .arg("-v")
        .arg("--json")
        .arg(backup_repo)
        .arg(source_path)
        .output();

    match output {
        Ok(message) => Ok(String::from_utf8_lossy(&message.stdout).to_string()),
        Err(_error) => Err("Unable to run BorgBackup. Something went wrong!".to_string()),
    }
}
