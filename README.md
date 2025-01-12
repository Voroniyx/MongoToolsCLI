# MongoTools

## 1) Cron Feature

### Environment Vars:
- `MONGOTOOLS_CRON_JOB`
  - Beispiel: `0 2 * * *` -> 'At 02:00.' 
- `MONGOTOOLS_CONNECTION_STRING`
  - Beispiel: `mongodb://<db_user_name>:<db_user_pwd>@<ip>:<port>/<db_name>?retryWrites=true&w=majority`

### Beispiel Ausführung:
Windows:
````bash
set MONGOTOOLS_CRON_JOB=0 2 * * * && set MONGOTOOLS_CONNECTION_STRING=<connection_string>  && MongoToolsCLI.exe
````

## 2) CLI
Run:
`MongoToolsCLI.exe`

> [!NOTE]
> Aktuell kann man nur backups über den CLI erstellen 

Einfach den Stritten folgen.
