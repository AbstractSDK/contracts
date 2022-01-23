from unicodedata import name
from terra_sdk.core.bank import MsgSend
from terra_sdk.core.auth import StdFee
from terra_sdk.core.wasm import MsgStoreCode, MsgInstantiateContract, MsgExecuteContract
import base64
import json

import pathlib
import sys
from typing import List

from cw_os.contracts.manager import *
from cw_os.contracts.treasury import *
from cw_os.contracts.version_control import *
from cw_os.contracts.os_factory import *
from terra_sdk.core.coins import Coin
from cw_os.deploy import get_deployer

mnemonic = "man goddess right advance aim into sentence crime style salad enforce kind matrix inherit omit entry brush never flat strategy entire outside hedgehog umbrella"

# deployer = get_deployer(mnemonic=mnemonic, chain_id="columbus-5", fee=None)
deployer = get_deployer(mnemonic=mnemonic, chain_id="bombay-12", fee=None)
# deployer = get_deployer(mnemonic=mnemonic, chain_id="localterra", fee=None)

version_control = VersionControlContract(deployer)
manager = OSManager(deployer)
treasury = TreasuryContract(deployer)
factory = OsFactoryContract(deployer)

create_os = False


# version_control.add_module_code_id(name="Manager", version= "v0.1.0",code_id= version_control.get("manager", True))
if create_os:
    # version_control.set_admin(deployer.wallet.key.acc_address)
    factory.create_os()

latest_os = int(factory.query_config()["os_id_sequence"]) - 1
os_address = version_control.query_os_address(latest_os)
deployer.store_contract_addr("manager", os_address)

version_control.query_code_id("Terraswap", "v0.1.0")

# TODO: add contract_ids to version_control

version_control.query_enabled_modules(latest_os)

treasury_addr = manager.query_modules(modules=["Treasury"])["modules"][0][1]
manager.query_modules(modules=["Treasury","Terraswap"])

terraswap_init_msg =  {
            "treasury_address": str(treasury_addr),
            "trader": deployer.wallet.key.acc_address,
            "memory_addr": deployer.wallet.key.acc_address
        }
# manager.update_config()
manager.query_os_config()

manager.add_internal_dapp("Terraswap", terraswap_init_msg)