```
cd /home/arch/Project/test/anthill-windsurf/infra/docker_compose
docker compose up -d
```
database migration
```
cd /home/arch/Project/test/anthill-windsurf
sqlx migrate run
```
backend service 
```
cd /home/arch/Project/test/anthill-windsurf
cargo run --bin user-service
```

inventory service
```
cd /home/arch/Project/test/anthill-windsurf
PORT=8001 cargo run --bin inventory-service
```
frontend service
```
cd /home/arch/Project/test/anthill-windsurf/frontend
bun run dev
```
