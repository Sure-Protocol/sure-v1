#!/bin/bash
#(()) Setup Solana environment (())
trap 'killall' INT
killall() {
    trap '' INT TERM     # ignore INT and TERM while shutting down
    echo "**** Shutting down... ****"     # added double quotes
    kill -TERM 0         # fixed order, send TERM not INT
    wait
    echo DONE
}
# Remove test ledger
rm -rf test-ledger

# Start up solana validator 
solana-test-validator & 

sleep 2

# Airdrop tokens to address
solana airdrop 2 rYhoVCsVF8dahDpAYUZ9sDygLbhoVgRcczMxnQhWWjg 
solana airdrop 2 Acyq4k7tJ38DyG4kppEEUF9AH1Cuiw7cGCfBuoEh8zH9 

# 
cp /Users/kristofferhovlandberg/.config/solana/sureLJ8UXoy3WF3Dk6Hy1ak8DscZkmHvv1hprvhVCxB.json target/deploy/sure_pool-keypair.json

# Deploy anchor program 
anchor deploy --provider.cluster localnet 

