"use client";

import EmptyStateCustom from "@/components/EmptyStateCustom";
import { useColorModeValue } from "@/components/ui/color-mode";
import { Outcome, TradeType } from "@/generated/grpc_service_types/markets";
import { MarketGetters } from "@/utils/interactions/dataGetter";
import {
  Box,
  Table,
  Card,
  CardHeader,
  CardBody,
  Heading,
  Text,
  Badge,
  Avatar,
  HStack,
  VStack,
  Icon,
  SimpleGrid,
  Stat,
  StatLabel,
  Separator,
  Pagination,
  IconButton,
  ButtonGroup,
} from "@chakra-ui/react";
import { useQuery } from "@tanstack/react-query";
import {
  TrendingUp,
  Activity,
  Clock,
  LucideChevronRight,
  LucideChevronLeft,
} from "lucide-react";
import { useState } from "react";

function formatDate(dateString: string) {
  return new Date(dateString).toLocaleDateString("en-US", {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function formatPrice(price: number) {
  return `$${price.toFixed(2)}`;
}

function getTradeTypeColor(tradeType: TradeType) {
  switch (tradeType) {
    case TradeType.BUY:
      return "green";
    case TradeType.SELL:
      return "red";
    default:
      return "gray";
  }
}

function getOutcomeColor(outcome: Outcome) {
  return outcome === Outcome.YES ? "blue" : "orange";
}
type Props = {
  market_id: string;
};

export default function MarketTrades({ market_id }: Props) {
  const cardBg = useColorModeValue("white", "gray.800");
  const borderColor = useColorModeValue("gray.200", "gray.600");
  const hoverBg = useColorModeValue("gray.50", "gray.700");
  const [page, setPage] = useState(1);

  const { data } = useQuery({
    queryKey: ["marketTrades", market_id, page],
    queryFn: () =>
      MarketGetters.getMarketTrades({
        marketId: market_id,
        page,
        pageSize: 20,
      }),
  });

  if (!data) {
    return (
      <EmptyStateCustom
        title="No Trades Found"
        description="There are no trades available for this market."
      />
    );
  }
  const tradesData = data.trades;

  const totalTrades = tradesData.length;
  const totalVolume = tradesData.reduce(
    (sum, trade) => sum + trade.price * trade.quantity,
    0,
  );
  const avgPrice =
    tradesData.reduce((sum, trade) => sum + trade.price, 0) / tradesData.length;
  const buyTrades = tradesData.filter(
    (trade) => trade.tradeType === TradeType.BUY,
  ).length;

  return (
    <Box>
      <VStack gap={6} align="stretch">
        {/* Header */}
        <Card.Root bg={cardBg} borderRadius="lg" py={4}>
          <Card.Header>
            <HStack justify="space-between" align="center">
              <VStack align="start" gap={1}>
                <Heading size="lg">Market Trades</Heading>
                <Text color="gray.500" fontSize="sm">
                  Recent trading activity
                </Text>
              </VStack>
              <Icon boxSize={6} color="blue.500">
                <Activity />
              </Icon>
            </HStack>
          </Card.Header>
        </Card.Root>

        {/* Summary Stats */}
        <SimpleGrid columns={{ base: 2, md: 4 }} gap={4}>
          <Card.Root bg={cardBg} borderRadius="lg">
            <Card.Body p={4}>
              <Stat.Root>
                <Stat.Label fontSize="xs" color="gray.500">
                  Total Trades
                </Stat.Label>
                <Stat.HelpText fontSize="2xl">{totalTrades}</Stat.HelpText>
              </Stat.Root>
            </Card.Body>
          </Card.Root>

          <Card.Root bg={cardBg} borderRadius="lg">
            <Card.Body p={4}>
              <Stat.Root>
                <Stat.Label fontSize="xs" color="gray.500">
                  Total Volume
                </Stat.Label>
                <Stat.HelpText fontSize="2xl">
                  {formatPrice(totalVolume)}
                </Stat.HelpText>
              </Stat.Root>
            </Card.Body>
          </Card.Root>

          <Card.Root bg={cardBg} borderRadius="lg">
            <Card.Body p={4}>
              <Stat.Root>
                <Stat.Label fontSize="xs" color="gray.500">
                  Avg Price
                </Stat.Label>
                <Stat.ValueText fontSize="2xl">
                  {formatPrice(avgPrice || 0)}
                </Stat.ValueText>
              </Stat.Root>
            </Card.Body>
          </Card.Root>

          <Card.Root bg={cardBg} borderRadius="lg">
            <Card.Body p={4}>
              <Stat.Root>
                <StatLabel fontSize="xs" color="gray.500">
                  Buy Orders
                </StatLabel>
                <Stat.HelpText fontSize="2xl">{buyTrades}</Stat.HelpText>
                <Stat.HelpText color="green.500">
                  <Icon mr={1}>
                    <TrendingUp />
                  </Icon>
                  {((buyTrades / totalTrades) * 100).toFixed(0)}%
                </Stat.HelpText>
              </Stat.Root>
            </Card.Body>
          </Card.Root>
        </SimpleGrid>

        {/* Trades Table */}
        <Card.Root bg={cardBg} borderRadius="lg" overflow="hidden">
          <Card.Body p={0}>
            <Box>
              <Table.Root variant="outline" size="md">
                <Table.Header bg={hoverBg}>
                  <Table.Row>
                    <Table.ColumnHeader>Trader</Table.ColumnHeader>
                    <Table.ColumnHeader>Type</Table.ColumnHeader>
                    <Table.ColumnHeader>Outcome</Table.ColumnHeader>
                    <Table.ColumnHeader>Price</Table.ColumnHeader>
                    <Table.ColumnHeader>Quantity</Table.ColumnHeader>
                    <Table.ColumnHeader>Total</Table.ColumnHeader>
                    <Table.ColumnHeader>Time</Table.ColumnHeader>
                  </Table.Row>
                </Table.Header>
                <Table.Body>
                  {data.trades.map((trade) => (
                    <Table.Row key={trade.id} _hover={{ bg: hoverBg }}>
                      <Table.Cell>
                        <HStack gap={3}>
                          <Avatar.Root size="sm">
                            <Avatar.Fallback name={trade.name} />
                          </Avatar.Root>
                          <VStack align="start" gap={0}>
                            <Text fontWeight="medium" fontSize="sm">
                              {trade.name}
                            </Text>
                            <Text color="gray.500" fontSize="xs">
                              {trade.email}
                            </Text>
                          </VStack>
                        </HStack>
                      </Table.Cell>
                      <Table.Cell>
                        <Badge
                          colorScheme={getTradeTypeColor(trade.tradeType)}
                          variant="solid"
                          borderRadius="full"
                          px={2}
                          py={1}
                          fontSize="xs"
                        >
                          {TradeType[trade.tradeType]}
                        </Badge>
                      </Table.Cell>
                      <Table.Cell>
                        <Badge
                          colorScheme={getOutcomeColor(trade.outcome)}
                          variant="outline"
                          borderRadius="full"
                          px={2}
                          py={1}
                          fontSize="xs"
                        >
                          {trade.outcome}
                        </Badge>
                      </Table.Cell>
                      <Table.Cell fontWeight="medium">
                        {formatPrice(trade.price)}
                      </Table.Cell>
                      <Table.Cell fontWeight="medium">
                        {trade.quantity}
                      </Table.Cell>
                      <Table.Cell fontWeight="bold" color="blue.500">
                        {formatPrice(trade.price * trade.quantity)}
                      </Table.Cell>
                      <Table.Cell>
                        <HStack gap={1}>
                          <Icon boxSize={3} color="gray.400">
                            <Clock />
                          </Icon>
                          <Text fontSize="sm" color="gray.500">
                            {formatDate(trade.createdAt)}
                          </Text>
                        </HStack>
                      </Table.Cell>
                    </Table.Row>
                  ))}
                </Table.Body>
              </Table.Root>
            </Box>
          </Card.Body>
        </Card.Root>

        {/* Mobile-friendly Cards View (Alternative) */}
        <Card.Root
          bg={cardBg}
          borderRadius="lg"
          display={{ base: "block", md: "none" }}
        >
          <CardHeader>
            <Heading size="md">Trade History (Mobile)</Heading>
          </CardHeader>
          <CardBody>
            <VStack gap={4} align="stretch">
              {tradesData.map((trade) => (
                <Box
                  key={trade.id}
                  p={4}
                  border="1px"
                  borderColor={borderColor}
                  borderRadius="lg"
                  _hover={{ shadow: "md" }}
                >
                  <VStack gap={3} align="stretch">
                    <HStack justify="space-between">
                      <HStack gap={3}>
                        <Avatar.Root size="sm">
                          <Avatar.Fallback name={trade.name} />
                        </Avatar.Root>
                        <VStack align="start" gap={0}>
                          <Text fontWeight="medium" fontSize="sm">
                            {trade.name}
                          </Text>
                          <Text color="gray.500" fontSize="xs">
                            {formatDate(trade.createdAt)}
                          </Text>
                        </VStack>
                      </HStack>
                      <VStack align="end" gap={0}>
                        <Text fontWeight="bold" color="blue.500">
                          {formatPrice(trade.price * trade.quantity)}
                        </Text>
                        <Text fontSize="xs" color="gray.500">
                          Total
                        </Text>
                      </VStack>
                    </HStack>

                    <Separator />

                    <HStack justify="space-between">
                      <HStack gap={2}>
                        <Badge
                          colorScheme={getTradeTypeColor(trade.tradeType)}
                          variant="solid"
                          size="sm"
                        >
                          {TradeType[trade.tradeType]}
                        </Badge>
                        <Badge
                          colorScheme={getOutcomeColor(trade.outcome)}
                          variant="outline"
                          size="sm"
                        >
                          {Outcome[trade.outcome]}
                        </Badge>
                      </HStack>
                      <HStack gap={4} fontSize="sm">
                        <Text>
                          <Text as="span" color="gray.500">
                            Price:
                          </Text>{" "}
                          {formatPrice(trade.price)}
                        </Text>
                        <Text>
                          <Text as="span" color="gray.500">
                            Qty:
                          </Text>{" "}
                          {trade.quantity}
                        </Text>
                      </HStack>
                    </HStack>
                  </VStack>
                </Box>
              ))}
            </VStack>
          </CardBody>
        </Card.Root>
        <Pagination.Root
          pageSize={data.pageInfo?.pageSize}
          page={data.pageInfo?.page}
          count={
            (data.pageInfo?.totalPages || 0) * (data.pageInfo?.pageSize || 0)
          }
        >
          <ButtonGroup variant="ghost" size="sm" wrap="wrap">
            <Pagination.PrevTrigger asChild>
              <IconButton
                onClick={() => setPage((prev) => Math.max(prev - 1, 1))}
              >
                <LucideChevronLeft />
              </IconButton>
            </Pagination.PrevTrigger>

            <Pagination.Items
              render={(page) => (
                <IconButton
                  onClick={() => setPage(page.value)}
                  variant={{ base: "ghost", _selected: "outline" }}
                >
                  {page.value}
                </IconButton>
              )}
            />

            <Pagination.NextTrigger asChild>
              <IconButton onClick={() => setPage((prev) => prev + 1)}>
                <LucideChevronRight />
              </IconButton>
            </Pagination.NextTrigger>
          </ButtonGroup>
        </Pagination.Root>
      </VStack>
    </Box>
  );
}
