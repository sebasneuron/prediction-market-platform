"use client";

import { useQuery } from "@tanstack/react-query";
import { Box, Flex, Separator, Text } from "@chakra-ui/react";

import { OrderGetters } from "@/utils/interactions/dataGetter";

type Props = {
  marketId: string;
};

const HoldingsInfoClient = ({ marketId }: Props) => {
  const { isLoading, data } = useQuery({
    queryKey: ["marketOrders", marketId],
    queryFn: () => OrderGetters.getUserOrdersByMarket(marketId, 1, 10),
  });
  const yesHoldings = isLoading
    ? "--"
    : Number(data?.holdings.yes).toFixed(3) || "0";
  const noHoldings = isLoading
    ? "--"
    : Number(data?.holdings.no).toFixed(3) || "0";
  return (
    <Box>
      <Flex align="center" gap={3}>
        <Text fontSize="sm" fontWeight="semibold" color="gray.600">
          Holdings
        </Text>
        <Separator orientation="vertical" h="20px" />
        <Flex align="center" gap={1}>
          <Box w={2} h={2} bg="green.500" borderRadius="full" />
          <Text fontSize="sm" fontWeight="bold" color="green.600">
            {yesHoldings}
          </Text>
        </Flex>
        <Flex align="center" gap={1}>
          <Box w={2} h={2} bg="red.500" borderRadius="full" />
          <Text fontSize="sm" fontWeight="bold" color="red.500">
            {noHoldings}
          </Text>
        </Flex>
      </Flex>
    </Box>
  );
};

export default HoldingsInfoClient;
