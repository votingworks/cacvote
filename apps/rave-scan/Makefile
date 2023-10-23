APP := rave-scan

ALL: build

clean: clean-frontend clean-backend

clean-frontend:
	@echo "🧹 Cleaning frontend…"
	@cd frontend && dx clean

clean-backend:
	@echo "🧹 Cleaning backend…"
	@cd backend && cargo clean

build: build-frontend build-backend

build-frontend:
	@echo "🛠️ Building frontend…"
	@cd frontend && dx build --release

build-backend:
	@echo "🛠️ Building backend…"
	@cd backend && cargo build --release

dist: build
	@echo "📦 Packaging application…"
	@rm -rf dist && mkdir dist
	@cp -r frontend/dist dist/public
	@cp ../../target/release/$(APP)-backend dist/$(APP)
	@echo "- \e[34;4mdist/public\e[0m: frontend assets"
	@echo "- \e[34;4mdist/$(APP)\e[0m: application binary"

run: dist
	@echo "🚀 Running application in production mode…"
	@cd dist && ./$(APP)

reset-db:
	@echo "🗑️ Resetting database…"
	@cd backend && cargo sqlx database reset --source db/migrations
	@echo "✅ Database reset"