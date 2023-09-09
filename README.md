# Touls

## BorgFlux

BorgBackup + InfluxDB

### Variables

You need to add a configuration file in order to connect to an InfluxDB instance and to backup
a specific path:

```sh
touls borgflux -c config_file.toml
```

- ``influx_url`` - URL to InfluxDB
- ``influx_token`` - API token for InfluxDB
- ``influx_org`` - Organization to use for InfluxDB
- ``influx_bucket`` - InfluxDB bucket to send data to
- ``hostname`` - custom host name (used for points within InfluxDB)
- ``borg_repository`` - Borg repository for backup data
- ``borg_source_path`` - local repository to create a backup of

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

## Wakey

Send magic packets to start WOL enabled machines:

```sh
touls wake_on_lan aa:bb:cc:dd:ee:ff
```
