from substrateinterface import SubstrateInterface, Keypair
from substrateinterface.contracts import ContractCode
import os
import time

node = SubstrateInterface(url="ws://127.0.0.1:9944", ss58_format=42)
print(f"Connected to: {node.chain} at block {node.get_block_number(node.get_chain_head())}")

alice = Keypair.create_from_uri("//Alice")
bob = Keypair.create_from_uri("//Bob")

contract_dir = os.path.expanduser("~/equilibria/contracts/stability_pool/target/ink")
metadata_file = os.path.join(contract_dir, "stability_pool.json")
wasm_file = os.path.join(contract_dir, "stability_pool.wasm")

with open(wasm_file, "rb") as f:
    wasm_bytes = f.read()

print(f"WASM size: {len(wasm_bytes)} bytes")

code = ContractCode.create_from_contract_files(
    metadata_file=metadata_file,
    wasm_file=wasm_file,
    substrate=node
)

constructor_data = code.metadata.generate_constructor_data(
    name="new",
    args={"keeper": bob.ss58_address, "max_deploy_pct": 20}
)

constructor_bytes = constructor_data.data

# First dry-run to get gas estimate
print("Getting fee info...")
call = node.compose_call(
    call_module="Contracts",
    call_function="instantiate_with_code",
    call_params={
        "endowment": 0,
        "gas_limit": 500000000000,
        "code": wasm_bytes,
        "data": constructor_bytes,
        "salt": b"equilibria01"
    }
)

payment_info = node.get_payment_info(call=call, keypair=alice)
print(f"Payment info: {payment_info}")

extrinsic = node.create_signed_extrinsic(call=call, keypair=alice)
print("Submitting...")
result = node.submit_extrinsic(extrinsic, wait_for_inclusion=False)
print(f"Extrinsic hash: {result.extrinsic_hash}")

print("Waiting 20 seconds...")
time.sleep(20)

contracts = node.query_map('Contracts', 'ContractInfoOf')
count = 0
for addr, info in contracts:
    print(f"✅ Contract: {addr.value}")
    count += 1
if count == 0:
    print("Still no contracts found")
