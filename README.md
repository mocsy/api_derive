### Start the db
Local Cluster mode:
```bash
export HOST_IP=<your_host_ip_here>
docker volume create arangodb1
docker run -it --name=adb1 --rm -p 8528:8528 \
-v arangodb1:/data \
-v /var/run/docker.sock:/var/run/docker.sock \
arangodb/arangodb-starter \
--starter.address=$HOST_IP \
--starter.local
```

An alternative way to start a single instance to test if something would work in non-cluster mode as well.
``` bash
docker run -e ARANGO_ROOT_PASSWORD=test_password -e ARANGODB_OVERRIDE_DETECTED_TOTAL_MEMORY=2G -e ARANGODB_OVERRIDE_DETECTED_NUMBER_OF_CORES=2 -p 8529:8529 -d arangodb
```

Go to <http://localhost:8529> and setup your test db.
```
DB_NAME=test_dev
DB_CONN=http://localhost:8529
ARANGO_PASSWORD="test_dev_pw"
ARANGO_USER_NAME=test_dev
```

Stop and remove the docker container. Note: if you keep the volume, you keep the db data.
```bash
docker container ls -a
docker container stop [container_id]
docker container rm [container_id]
```

### Setup test data
- Naivgate to http://localhost:8529/
- Create a new db and user with password
- Complete .env with test data
- Fill the DB
