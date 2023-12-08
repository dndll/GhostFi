USER=ghostfi.testnet
if [ ! -z "$1" ]; then
  USER=$1
fi

CONTRACT=ghostfi.testnet
if [ ! -z "$2" ]; then
  CONTRACT=$2
fi

ENVIRONMENT=testnet
if [ ! -z "$3" ]; then
  ENVIRONMENT=$2
fi

SIGNER=ghostfi.testnet
if [ ! -z "$4" ]; then
  SIGNER=$3
fi

near contract call-function as-transaction \
  $CONTRACT register \
  json-args "{\"user\":\"$USER\"}" \
  prepaid-gas '100.0 Tgas' \
  attached-deposit '0 NEAR' \
  sign-as $SIGNER \
  network-config $ENVIRONMENT sign-with-keychain send
