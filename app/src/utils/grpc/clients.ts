import { GrpcWebFetchTransport } from "@protobuf-ts/grpcweb-transport";

import { MarketServiceClient } from "@/generated/grpc_service_types/markets.client";
import { PriceServiceClient } from "@/generated/grpc_service_types/price.client";

const transport = new GrpcWebFetchTransport({
  baseUrl: process.env.NEXT_PUBLIC_GRPC_API_URL || "http://localhost:5010",
  meta: {
    "Access-Control-Allow-Origin": "*",
    "some-random-shit": "Admin",
  },
}) as never;

const marketServiceClient = new MarketServiceClient(transport);
const priceServiceClient = new PriceServiceClient(transport);

export { marketServiceClient, priceServiceClient };
