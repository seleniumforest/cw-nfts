import * as dotenv from 'dotenv'
import { Secp256k1HdWallet } from "@cosmjs/launchpad";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { GasPrice } from "@cosmjs/stargate";
dotenv.config();

(async () => {
    const wallet = await Secp256k1HdWallet.fromMnemonic(process.env.PRIVATE!, { prefix: "sei" });
    const [{ address }] = await wallet.getAccounts();

    const client = await SigningCosmWasmClient.connectWithSigner(
        process.env.RPC!,
        wallet,
        { gasPrice: GasPrice.fromString("0.1usei") }
    );

    let result = await client.execute(
        address,
        process.env.CONTRACT_ADDR!,
        {
            mint: {
                extension: {},
                owner: address,
                token_uri: "https://alpaca.com/alpaca.jpg"
            }
        },
        "auto", "", [{ denom: "usei", amount: "10000" }]);

    console.log(result.transactionHash);
})();