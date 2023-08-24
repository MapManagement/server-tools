# Touls

## BorgFlux

BorgBackup + InfluxDB

### Variables

- ``INFLUX_URL`` - URL to InfluxDB
- ``INFLUX_TOKEN`` - API token for InfluxDB
- ``INFLUX_ORG`` - Organization to use for InfluxDB
- ``INFLUX_BUCKET`` - InfluxDB bucket to send data to
- ``HOST`` - custom host name (used for points within InfluxDB)
- ``BORG_REPOSITORY`` - Borg repository for backup data
- ``BORG_SOURCE_PATH`` - local repository to create a backup of

### Measurements

#### ``backup_data``

- Tags
    - host
    - backup_name
    - encryption
    - repo_location
- Fields
    - duration
    - compressed_size
    - deduplicated_size
    - number_of_files
    - original_size

#### ``backup_start`` / ``backup_end``

- Tags
    - host
    - repository
    - source_path
- Fields
    - dummy

#### ``backup_error``

- Tags
    - host
    - repository
    - source_path
- Fields
    - error
