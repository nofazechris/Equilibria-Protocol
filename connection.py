python3 -c "
from substrateinterface import SubstrateInterface
node = SubstrateInterface(url='ws://127.0.0.1:9944', ss58_format=42)
print('Connected, block:', node.get_block_number(node.get_chain_head()))
"