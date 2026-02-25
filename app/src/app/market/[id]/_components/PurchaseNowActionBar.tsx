"use client";

import React, { useEffect, useRef, useState } from "react";
import {
  Box,
  Button,
  Flex,
  Tabs,
  useDisclosure,
  Portal,
  Select,
  CloseButton,
  createListCollection,
} from "@chakra-ui/react";

import TradeForm from "./TradeForm";
import { MarketPrice } from "@/generated/grpc_service_types/markets";

type Props = {
  market_id: string;
  marketPrice: MarketPrice;
};

const PurchaseNowActionBar = ({ market_id, marketPrice }: Props) => {
  const { open: isOpen, onToggle } = useDisclosure();
  const [orderType, setOrderType] = useState<"market" | "limit">("market");

  const ctnRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (ctnRef.current && !ctnRef.current.contains(event.target as Node)) {
        onToggle();
      }
    };

    document.addEventListener("click", handleClickOutside);
    return () => {
      document.removeEventListener("click", handleClickOutside);
    };
  }, []);

  return (
    <Box
      position="fixed"
      left={0}
      right={0}
      bottom={5}
      zIndex={10}
      width={isOpen ? "400px" : "140px"}
      minHeight="80px"
      mx="auto"
      overflow="hidden"
      transition="all 0.2s ease-in-out"
    >
      {!isOpen ? (
        <Button
          onClick={onToggle}
          width="100%"
          size="lg"
          bg="blue.subtle/50"
          backdropBlur="md"
          backdropFilter="blur(10px)"
          variant="outline"
          rounded="full"
        >
          Trade Now
        </Button>
      ) : (
        <Box
          bg="gray.subtle/50"
          backdropBlur="md"
          backdropFilter="blur(10px)"
          boxShadow="0 -2px 8px rgba(0,0,0,0.08)"
          px={6}
          py={4}
          borderRadius="xl"
          minHeight="250px"
          _hover={{ boxShadow: "0 -4px 12px rgba(0,0,0,0.1)" }}
          ref={ctnRef}
        >
          <Tabs.Root defaultValue="buy">
            <Tabs.List
              justifyContent={"space-between"}
              display="flex"
              alignItems="center"
              gap={2}
            >
              <Flex>
                <Tabs.Trigger value="buy">Buy</Tabs.Trigger>
                <Tabs.Trigger value="sell">Sell</Tabs.Trigger>
              </Flex>
              <Flex gap={2}>
                <Select.Root
                  collection={orderTypes}
                  size="sm"
                  width="100px"
                  defaultValue={[orderType]}
                  onValueChange={(v) =>
                    setOrderType(v.value[0] as typeof orderType)
                  }
                >
                  <Select.HiddenSelect />
                  <Select.Control>
                    <Select.Trigger border={"none"}>
                      <Select.ValueText placeholder="Type" />
                    </Select.Trigger>
                    <Select.IndicatorGroup>
                      <Select.Indicator />
                    </Select.IndicatorGroup>
                  </Select.Control>
                  <Portal>
                    <Select.Positioner>
                      <Select.Content bg="gray.50">
                        {orderTypes.items.map((orderType) => (
                          <Select.Item item={orderType} key={orderType.value}>
                            {orderType.label}
                            <Select.ItemIndicator />
                          </Select.Item>
                        ))}
                      </Select.Content>
                    </Select.Positioner>
                  </Portal>
                </Select.Root>
                <CloseButton onClick={onToggle} />
              </Flex>
            </Tabs.List>
            <Tabs.Content value="buy">
              <TradeForm
                mode="buy"
                orderType={orderType}
                market_id={market_id}
                marketPrice={marketPrice}
              />
            </Tabs.Content>
            <Tabs.Content value="sell">
              <TradeForm
                mode="sell"
                orderType={orderType}
                market_id={market_id}
                marketPrice={marketPrice}
              />
            </Tabs.Content>
          </Tabs.Root>
        </Box>
      )}
    </Box>
  );
};

export default PurchaseNowActionBar;

const orderTypes = createListCollection({
  items: [
    { label: "Limit", value: "limit" },
    { label: "Market", value: "market" },
  ],
});
