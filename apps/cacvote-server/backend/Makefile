ALL: build

clean:
	@echo "🧹 Cleaning…"
	@cargo clean

build:
	@echo "🛠️ Building…"
	@cargo build --release

dist: build
	@echo "📦 Packaging application…"
	@rm -rf dist && mkdir dist
	@cp ../../../target/release/cacvote-server dist/cacvote-server
	@echo "- \e[34;4mdist/cacvote-server\e[0m: application binary"

run:
	@echo "🚀 Running application in production mode…"
	@cd dist && ./cacvote-server

reset-db:
	@echo "🗑️ Resetting database…"
	@cargo sqlx database reset --source db/migrations
	@echo "✅ Database reset"
