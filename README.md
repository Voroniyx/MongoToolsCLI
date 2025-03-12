# MongoTools

## 1) Cron Feature
Executes the backup function based on the cronJob to automate the process of creating backups of a database.
Backups are stored in a `.tar.gz` archive.
## 2) CLI
Possibility of manually creating a backup or restoring a database from a `.tar.gz` archive created with the cronjob function or a manual backup
## 3) Config
It is possible to create a JSON Config, which could be helpful if you often have to create manual backups. 
However, it is actually made for the cron mode to replace the ENVIRONMENT variables. If `forceCli` is set to true, you will be asked whether you want to make a restore or a backup and then the respective options are read from the config.

> [!NOTE]
> The cron expression must be 6 characters long like the Spring Cron expressions

```json
{
  "cronJobExpression": String,
  "connectionString": String,
  "forceCli": Boolean,
  "targzPath": String
}
```

`connectionString` example:
```
# srv is supported too
mongodb://<db_user_name>:<db_user_pwd>@<ip>:<port>/<db_name>?retryWrites=true&w=majority
```


`cronJobExpression` example:
```
0 * * * * * 
```
*Would create a backup every minute*

## Docker
```
docker run --name <containername> -d -v /path/to/config.json:/app/config.json -v /path/to/backupfolder:/app/<targzPath-outputpath> ghcr.io/voroniyx/mongo-tools-cli
```
On Windows, the paths may have to start and end in `"`
