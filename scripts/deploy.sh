CONTRACT=ghostfi.testnet
if [ ! -z "$1" ]; then
  USER=$1
fi

PROVER=ghostfi.testnet
if [ ! -z "$2" ]; then
  PROVER=$2
fi

ENVIRONMENT=testnet
if [ ! -z "$3" ]; then
  ENVIRONMENT=$2
fi

SIGNER=ghostfi.testnet
if [ ! -z "$4" ]; then
  SIGNER=$3
fi

near contract \
  deploy $CONTRACT \
  use-file ./target/wasm32-unknown-unknown/release/ghostfi.wasm \
  with-init-callinitialize \
  json-args "{\"prover\":\"$PROVER\"}" \
  prepaid-gas '100.0 Tgas' \
  attached-deposit '0.00 NEAR' \
  sign-as $SIGNER \
  network-config $ENVIRONMENT sign-with-keychain send

