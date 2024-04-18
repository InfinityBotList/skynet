all:
	cargo build --release
restartwebserver:
	cargo sqlx prepare
	make all
	make restartwebserver_nobuild

restartwebserver_nobuild:
	sudo systemctl stop skynet
	sleep 3 # Give time for it to stop
	cp -v target/release/skynet skynet
	sudo systemctl start skynet
