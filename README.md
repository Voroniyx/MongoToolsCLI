# MongoTools

## 1) Cron Feature

### Environment Vars:
- `MONGOTOOLS_CRON_JOB`
  - Beispiel: `0 * * * * *` -> 'Every Minute' 
  > ![NOTE]
  > Die Cron-Expression muss 6 stellen lang sein
- `MONGOTOOLS_CONNECTION_STRING`
  - Beispiel: `mongodb://<db_user_name>:<db_user_pwd>@<ip>:<port>/<db_name>?retryWrites=true&w=majority`

### Beispiel Ausführung:
Windows:
```bash
> set MONGOTOOLS_CRON_JOB="0 * * * * *" # every minute
> set MONGOTOOLS_CONNECTION_STRING="<connection_string>"
> MongoToolsCLI.exe
```

## 2) CLI
Run:
`MongoToolsCLI.exe`

Einfach den Stritten folgen.
