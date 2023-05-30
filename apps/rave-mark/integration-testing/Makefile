APP := ..
export PIPENV_VENV_IN_PROJECT=1
export RAVE_MARK_WORKSPACE=/tmp/rave-mark-integration-testing
export BASE_URL=http://localhost:3000/
export IS_INTEGRATION_TEST=true

clean-downloads:
	rm cypress/downloads/*

build-frontend:
	make -C $(APP)/frontend install; \
	make -C $(APP)/frontend build; \

build-backend:
	make -C $(APP)/backend install; \
	make -C $(APP)/backend build; \

build: build-frontend build-backend

run:
	rm -rf $(RAVE_MARK_WORKSPACE)
	mkdir $(RAVE_MARK_WORKSPACE)
	NODE_ENV=production VX_MACHINE_TYPE=rave-mark \
		pnpm concurrently -n frontend,backend --kill-others \
		"make -C $(APP)/frontend run" \
		"make -C $(APP)/backend run"

test:
	pnpm start-server-and-test start $(BASE_URL) cypress:run

test-watch:
	pnpm start-server-and-test start $(BASE_URL) cypress:open
