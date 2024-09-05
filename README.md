# thermite

A lightweight task worker in Rust

## Heroku Container Deployment Steps

1. Install Heroku CLI
2. Login to Heroku
3. Login to Heroku Container Registry

```bash
heroku login
heroku container:login
```

4. Create a Heroku app

```bash
heroku create thermite
```

5. Build the Docker image

```bash
docker build --platform linux/amd64 -t registry.heroku.com/thermite/worker .
```

6. Push the Docker image to Heroku Container Registry

```bash
docker push registry.heroku.com/thermite/worker
```

7. Set the stack to container

```bash
heroku stack:set container -a thermite
```

7. Release the Docker image

```bash
heroku container:release worker -a thermite
```

8. Open the Heroku app

```bash
heroku open -a thermite
```

9. View the logs

```bash
heroku logs --tail -a thermite
```
