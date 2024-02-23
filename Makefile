COMPOSE := docker compose -f docker-compose.yml
EXPOSED_PORTS := 50001 50002
BCLI := $(COMPOSE) exec -T -u blits bitcoind bitcoin-cli -regtest

.PHONY: up up-bitcoin up-electrs up-proxy down mine send create-wallet balance

up:
	@$(MAKE) down
	@$(MAKE) up-bitcoin
	@$(MAKE) create-wallet name=miner
	@$(MAKE) mine wallet=miner blocks=150
	@$(MAKE) up-electrs
	@$(MAKE) up-proxy

up-bitcoin:
	@make down
	@for port in $(EXPOSED_PORTS); do \
		if lsof -Pi :$$port -sTCP:LISTEN -t >/dev/null ; then \
			echo "port $$port is already bound, services can't be started"; \
 			exit 1; \
		fi \
	done
	@$(COMPOSE) up -d bitcoind
	until $(COMPOSE) logs bitcoind | grep 'Bound to'; do sleep 1; done

up-electrs:
	@$(COMPOSE) up -d electrs
	until $(COMPOSE) logs electrs | grep 'finished full compaction'; do sleep 1; done

up-proxy:
	@$(COMPOSE) up -d proxy
	until $(COMPOSE) logs proxy | grep 'App is running at http://localhost:3000'; do sleep 1; done

down:
	@$(COMPOSE) down -v
	@rm -rf storage/rgblib/*

create-wallet:
	@$(BCLI) createwallet $(name)

mine:
	@$(BCLI) -rpcwallet=$(wallet) -generate $(blocks)

send:
	@$(BCLI) -rpcwallet=$(wallet) sendtoaddress $(recipient) $(amount)
	@$(BCLI) -rpcwallet=$(wallet) -generate 4

balance:
	@$(BCLI) -rpcwallet=$(wallet) getbalance