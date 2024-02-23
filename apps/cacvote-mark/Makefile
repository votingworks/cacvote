APP := rave-mark

ALL: build

clean: clean-frontend clean-backend

clean-frontend:
	@echo "🧹 Cleaning frontend…"
	@cd frontend && pnpm clean

clean-backend:
	@echo "🧹 Cleaning backend…"
	@cd backend && pnpm clean

build: build-frontend build-backend

build-frontend:
	@echo "🛠️ Building frontend…"
	@cd frontend && pnpm build

build-backend:
	@echo "🛠️ Building backend…"
	@cd backend && pnpm build

dist:
	@echo "❌ Not implemented yet"
	@exit 1

run: build
	@echo "🚀 Running application in production mode…"
	@cd frontend && NODE_ENV=production pnpm concurrently --kill-others --names frontend,backend "pnpm --dir prodserver start" "pnpm --dir ../backend start"

reset-db:
	@echo "🗑️ Resetting database…"
	@test -d "${RAVE_MARK_WORKSPACE}" && rm -rf "${RAVE_MARK_WORKSPACE}" && echo "✅ Database reset" || echo "❌ Database not reset, no RAVE_MARK_WORKSPACE"