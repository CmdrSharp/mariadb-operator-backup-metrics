# MariaDB Operator - Backup Metrics

[MariaDB Operator](https://github.com/mariadb-operator/mariadb-operator) exposes a number of useful metrics for Galera, but none related to the scheduled backups. There needs to be a way to observe and monitor metrics related to backups, to verify that they were run successfully.

This tool watches the `Backup` CRD's from the MariaDB Operator, and exposes Prometheus-style metrics from the status field of these CRD's. The metric is labeled based on the name of the `Backup` CRD.

## Installation

Installation is done with [Helm](https://helm.sh/).

```bash
helm repo add mariadb-operator-backup-metrics https://cmdrsharp.github.io/mariadb-operator-backup-metrics
helm repo update
helm install mariadb-operator-backup-metrics mariadb-operator-backup-metrics/mariadb-operator-backup-metrics
```

## Metrics

Metrics are Prometheus-style, exposed over HTTP on the `/metrics` path.

The metrics exposed are `backup_last_run_status`, which is a `gauge` where `1` means successful and `0` means failed.  
`backup_last_run_timestamp` is a UNIX Timestamp of the last run backup.

```
# TYPE backup_last_run_status gauge
backup_last_run_status{reason="CronJobSucess",name="my-backup-name"} 1

# TYPE backup_last_run_timestamp gauge
backup_last_run_timestamp{name="my-backup-name"} 1724874096
```

> [!NOTE]  
> Since the MariaDB Operator uses `conditions` to convey state, we can only look at the _current state_. To avoid too much flappiness, the `backup_last_run_timestamp` will remain at its old value until a new successful backup has been run to completion. It does not update when new cron jobs are scheduled, for example. `backup_last_run_status` does however update to follow the progress of the backup.
