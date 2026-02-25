"use client";

import React, { useEffect, useState } from "react";
import {
  Box,
  Table,
  Badge,
  Text,
  Card,
  HStack,
  VStack,
  Heading,
  Icon,
} from "@chakra-ui/react";
import { useQuery } from "@tanstack/react-query";

import { MarketGetters } from "@/utils/interactions/dataGetter";

import {
  GetMarketBookResponse,
  OrderLevel,
} from "@/generated/grpc_service_types/markets";
import useSubscription from "@/hooks/useSubscription";
import { MarketBook } from "@/generated/service_types/ws_server/order_book";
import { formatPriceString } from "@/utils";
import { BarChart } from "lucide-react";

type Props = {
  tradeType: "yes" | "no";
  marketId: string;
};

const OrderBook = ({ tradeType, marketId }: Props) => {
  const { data } = useQuery({
    queryKey: ["orderBookData", marketId],
    queryFn: () => MarketGetters.getOrderBook(marketId),
  });
  const { messages } = useSubscription<MarketBook>(
    "/proto/proto_defs/ws_server/order_book.proto",
    "order_book.MarketBook",
    {
      payload: {
        type: "Subscribe",
        data: {
          channel: `order_book_update:${marketId}`,
        },
      },
    },
    false, // maintainPreviousMessages to false
  );
  const [orders, setOrders] = useState<OrderBookLevel[]>([]);

  useEffect(() => {
    if (data) {
      const allOrders = transformToOrderBookLevels(data, tradeType);
      setOrders(allOrders);
    }
  }, [data]);

  // websocket messages
  useEffect(() => {
    if (messages?.length) {
      const newOrders = messages.map((msg) => {
        return transformToOrderBookLevels(msg, tradeType);
      });
      setOrders(newOrders.flat());
    }
  }, [messages]);

  const buyOrders = orders
    .filter((o) => o.type === "buy")
    .sort((a, b) => b.price - a.price);
  const sellOrders = orders
    .filter((o) => o.type === "sell")
    .sort((a, b) => a.price - b.price);

  const mergedOrders = [...buyOrders, ...sellOrders];

  const buyTotalUsers = buyOrders.reduce((acc, order) => acc + order.users, 0);
  const sellTotalUsers = sellOrders.reduce(
    (acc, order) => acc + order.users,
    0,
  );

  return (
    <Box>
      <Card.Root borderRadius="lg" py={4} mb={6}>
        <Card.Header>
          <HStack justify="space-between" align="center">
            <VStack align="start" gap={1}>
              <Heading size="lg">
                {tradeType === "yes" ? "Yes Orders" : "No Orders"}
              </Heading>
              <Text color="gray.500" fontSize="sm">
                {tradeType === "yes"
                  ? "Orders for Yes side of the market"
                  : "Orders for No side of the market"}
              </Text>
            </VStack>
            <Icon boxSize={6} color="blue.500">
              <BarChart />
            </Icon>
          </HStack>
        </Card.Header>
      </Card.Root>
      <Table.ScrollArea borderWidth="1px" rounded="md">
        <Table.Root size="sm" stickyHeader bg="transparent" variant="outline">
          <Table.Header>
            <Table.Row bg="transparent">
              <Table.ColumnHeader>Trade {tradeType}</Table.ColumnHeader>
              <Table.ColumnHeader>Price</Table.ColumnHeader>
              <Table.ColumnHeader>Quantity</Table.ColumnHeader>
              <Table.ColumnHeader>Total</Table.ColumnHeader>
              <Table.ColumnHeader>Total Price (USD)</Table.ColumnHeader>
            </Table.Row>
          </Table.Header>

          <Table.Body>
            {mergedOrders.map((order, idx) => {
              const isSpreadRow = idx === buyOrders.length;
              const bestBuy = buyOrders[0]?.price ?? 0;
              const bestSell = sellOrders[0]?.price ?? 0;
              const spread = ((bestSell - bestBuy) * 100).toFixed(2);

              return (
                <React.Fragment key={`${order.type}-${order.price}-${idx}`}>
                  {isSpreadRow && (
                    <Table.Row key="spread-row" border="none">
                      <Table.Cell colSpan={4} textAlign="center" py={2}>
                        <Text fontWeight="bold" color="gray.600" fontSize="sm">
                          Spread: {spread}
                        </Text>
                      </Table.Cell>
                    </Table.Row>
                  )}
                  <Table.Row
                    border="none"
                    _hover={{
                      bg: order.type === "buy" ? "green.100/60" : "red.50/60",
                    }}
                  >
                    <Table.Cell padding={0} position="relative" height="100%">
                      <Box
                        position="absolute"
                        left={0}
                        top={0}
                        bottom={0}
                        width={getBarPercentage(
                          order.users,
                          order.type === "buy" ? buyTotalUsers : sellTotalUsers,
                        )}
                        bg={
                          order.type === "buy" ? "green.500/30" : "red.500/30"
                        }
                        height="100%"
                        zIndex={0}
                      />
                      <Box
                        position="relative"
                        zIndex={1}
                        display="flex"
                        alignItems="center"
                        height="25px"
                      >
                        {(idx === buyOrders.length - 1 ||
                          idx === buyOrders.length) && (
                          <Badge
                            bg={order.type === "buy" ? "green.500" : "red.500"}
                            color="white"
                            ml={4}
                          >
                            {order.type === "buy" ? "Bids" : "Asks"}
                          </Badge>
                        )}
                      </Box>
                    </Table.Cell>
                    <Table.Cell>{Number(order.price || 0) * 100}</Table.Cell>
                    <Table.Cell>{order.shares}</Table.Cell>
                    <Table.Cell>{order.total}</Table.Cell>
                    <Table.Cell>
                      {formatPriceString(
                        Number(order.price || 0) * 100 * order.shares,
                      )}
                    </Table.Cell>
                  </Table.Row>
                </React.Fragment>
              );
            })}
          </Table.Body>
        </Table.Root>
      </Table.ScrollArea>
    </Box>
  );
};

export default OrderBook;

type OrderBookLevel = {
  price: number;
  shares: number;
  total: number;
  users: number;
  type: "buy" | "sell";
};

function getBarPercentage(users: number, totalUsers: number): string {
  if (totalUsers === 0) return "0%";
  return ((users / totalUsers) * 100).toFixed(2) + "%";
}

function transformToOrderBookLevels(
  data?: GetMarketBookResponse | null,
  tradeType: "yes" | "no" = "yes",
): OrderBookLevel[] {
  const result: OrderBookLevel[] = [];
  if (!data) return result;

  const book = tradeType === "yes" ? data.yesBook : data.noBook;

  const processSide = (side: OrderLevel[], type: "buy" | "sell") => {
    let cumulative = 0;
    return side.map((entry) => {
      cumulative += entry.shares;
      return {
        price: entry.price,
        shares: entry.shares,
        total: cumulative,
        users: entry.users,
        type,
      };
    });
  };

  result.push(...processSide(book?.bids || [], "buy"));
  result.push(...processSide(book?.asks || [], "sell"));

  return result;
}
