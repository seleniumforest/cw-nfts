import { CosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import 'dotenv/config'

(async () => {
  const client = await CosmWasmClient.connect(
    process.env.RPC!
  );

  let result = await client.queryContractSmart(
    process.env.CONTRACT_ADDR!,
    {
      num_tokens: {}
    });

  console.log(result);
})();