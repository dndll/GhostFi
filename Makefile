contract:
	cargo build --release -p ghostfi --target wasm32-unknown-unknown

deploy-contract: contract
	scripts/deploy.sh

register:
	scripts/register_user.sh

