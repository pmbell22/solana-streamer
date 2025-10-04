#!/usr/bin/env python3
import json

# Read the IDL file
with open('dex-idl-parser/idls/raydium_amm_v4.json', 'r') as f:
    idl = json.load(f)

# Define discriminators for each instruction
discriminators = {
    'initialize2': [1],
    'deposit': [3],
    'withdraw': [4],
    'withdrawPnl': [7],
    'swapBaseIn': [9],
    'swapBaseOut': [11]
}

# Update instructions with discriminators
for instruction in idl['instructions']:
    if instruction['name'] in discriminators:
        instruction['discriminator'] = discriminators[instruction['name']]

# Write back the updated IDL
with open('dex-idl-parser/idls/raydium_amm_v4.json', 'w') as f:
    json.dump(idl, f, indent=2)

print("✅ Added discriminators to Raydium AMM V4 IDL:")
for name, disc in discriminators.items():
    print(f"  - {name}: {disc}")
