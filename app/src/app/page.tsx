"use client";

import { Container, HStack, Text } from "@chakra-ui/react";
import { useQueries } from "@tanstack/react-query";

import { MarketGetters } from "@/utils/interactions/dataGetter";
import TrendingMarketCard from "@/components/TrendingMarketCard";
import { MarketStatus } from "@/generated/grpc_service_types/markets";

export default function Home() {
  console.log("Variables", process.env);
  const [{ data }, { data: closedMarkets }] = useQueries({
    queries: [
      {
        queryKey: ["marketData", 1, 10, "open"],
        queryFn: () => MarketGetters.getMarketData(1, 10, MarketStatus.OPEN),
      },
      {
        queryKey: ["recentlyClosedMarkets", 1, 10, "closed"],
        queryFn: () => MarketGetters.getMarketData(1, 10, MarketStatus.CLOSED),
      },
    ],
  });
  return (
    <Container my={10}>
      <Text fontSize="2xl" fontWeight="bold" mb={4}>
        Trending Markets
      </Text>
      <HStack overflow="scroll">
        {data?.map((item) => <TrendingMarketCard market={item} />)}
      </HStack>

      <Text fontSize="2xl" fontWeight="bold" my={4}>
        Recently Closed
      </Text>
      <HStack overflow="scroll">
        {closedMarkets?.map((item) => <TrendingMarketCard market={item} />)}
      </HStack>
    </Container>
  );
}
