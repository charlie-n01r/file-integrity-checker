# Log Integrity Checker
A tool that verifies the integrity of application log files to detect changes and tampering.

## Usage:
```sh
./integrity-check init /var/log
# Hashes stored successfully.

./integrity-check check /var/log/syslog
# Status: Modified (Hash mismatch).

./integrity-check check /var/log/auth.log
# Status: Unmodified.

./integrity-check update /var/log/syslog
# Hash updated successfully.
```
