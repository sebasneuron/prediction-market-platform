"use client";

import { Box, Button, Flex, Text } from "@chakra-ui/react";
import { useState } from "react";

import MarketOrderForm from "./MarketOrderForm";
import LimitOrderForm from "./LimitOrderForm";
import { MarketPrice } from "@/generated/grpc_service_types/markets";
import { formatPriceString } from "@/utils";

type Props = {
  mode: "buy" | "sell";
  orderType: "market" | "limit";
  market_id: string;
  marketPrice: MarketPrice;
};

const TradeForm = ({ mode, orderType, market_id, marketPrice }: Props) => {
  const [stockMode, setStockMode] = useState<"yes" | "no">("yes");
  const yesPrice = formatPriceString(marketPrice.latestYesPrice);
  const noPrice = formatPriceString(marketPrice.latestNoPrice);

  return (
    <Box>
      <Flex gap={2} width="100%" justifyContent="space-between">
        <Button
          width="1/2"
          bg={stockMode === "yes" ? "green.600" : "gray.500"}
          _hover={{ bg: "green.600" }}
          onClick={() => setStockMode("yes")}
          py={6}
          rounded="lg"
        >
          Yes
          <Text fontSize="md" fontWeight="bold" color="white">
            {yesPrice}
          </Text>
        </Button>
        <Button
          width="1/2"
          bg={stockMode === "no" ? "red.600" : "gray.500"}
          _hover={{ bg: "red.600" }}
          onClick={() => setStockMode("no")}
          py={6}
          rounded="lg"
        >
          No
          <Text fontSize="md" fontWeight="bold" color="white">
            {noPrice}
          </Text>
        </Button>
      </Flex>

      {/* market / limit order form */}
      {orderType === "limit" ? (
        <LimitOrderForm
          mode={mode}
          stockMode={stockMode}
          market_id={market_id}
        />
      ) : (
        <MarketOrderForm
          mode={mode}
          stockMode={stockMode}
          market_id={market_id}
        />
      )}
    </Box>
  );
};

export default TradeForm;
