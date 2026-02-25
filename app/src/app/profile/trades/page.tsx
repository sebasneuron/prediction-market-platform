"use client";

import EmptyStateCustom from "@/components/EmptyStateCustom";
import { useColorModeValue } from "@/components/ui/color-mode";
import { UserGetters } from "@/utils/interactions/dataGetter";
import {
  Box,
  Card,
  CardBody,
  CardHeader,
  Heading,
  Text,
  Badge,
  Avatar,
  HStack,
  VStack,
  SimpleGrid,
  Flex,
  Icon,
  Stat,
  StatLabel,
  StatHelpText,
  Separator,
  Container,
  Pagination,
  ButtonGroup,
  IconButton,
} from "@chakra-ui/react";
import { useQuery } from "@tanstack/react-query";
import {
  TrendingUp,
  TrendingDown,
  Activity,
  DollarSign,
  BarChart3,
  ShoppingCart,
  LucideChevronLeft,
  LucideChevronRight,
} from "lucide-react";
import { useState } from "react";

function getTradeTypeColor(tradeType: string) {
  return tradeType.toLowerCase() === "buy" ? "green" : "red";
}

function getTradeTypeIcon(tradeType: string) {
  return tradeType.toLowerCase() === "buy" ? TrendingUp : TrendingDown;
}

function getOutcomeColor(outcome: string) {
  return outcome.toLowerCase() === "yes" ? "blue" : "orange";
}

function getStatusColor(status: string) {
  switch (status.toLowerCase()) {
    case "open":
      return "green";
    case "closed":
      return "orange";
    case "resolved":
      return "blue";
    default:
      return "gray";
  }
}

function formatPrice(price: string) {
  return `$${parseFloat(price).toFixed(2)}`;
}

function calculateTotal(price: string, quantity: string) {
  return formatPrice((parseFloat(price) * parseFloat(quantity)).toString());
}

export default function TradesDisplay() {
  const [page, setPage] = useState(1);
  const { data } = useQuery({
    queryKey: ["userTrades"],
    queryFn: () => UserGetters.getUserTrades(page, 10),
  });
  const cardBg = useColorModeValue("white", "gray.800");
  const borderColor = useColorModeValue("gray.200", "gray.600");
  const hoverBg = useColorModeValue("gray.50", "gray.700");
  const marketNameColor = useColorModeValue("gray.800", "white");
  const outcomeColor = useColorModeValue("gray.500", "gray.400");
  const typeColor = useColorModeValue("gray.500", "gray.400");
  const statusColor = useColorModeValue("gray.500", "gray.400");

  const totalColor = useColorModeValue("gray.500", "gray.400");

  if (!data || !data.data.trades || !data.data.page_info) {
    return (
      <EmptyStateCustom
        title="No Trades Found"
        description="You haven't made any trades yet. Start trading to see your activity here."
      />
    );
  }

  const tradesData = data.data.trades;
  const pageInfo = data.data.page_info;

  // Calculate summary stats
  const totalTrades = tradesData.length;
  const totalVolume = tradesData.reduce(
    (sum, trade) =>
      sum + parseFloat(trade.trade_price) * parseFloat(trade.trade_quantity),
    0,
  );
  const buyTrades = tradesData.filter(
    (trade) => trade.trade_type.toLowerCase() === "buy",
  ).length;
  const sellTrades = tradesData.filter(
    (trade) => trade.trade_type.toLowerCase() === "sell",
  ).length;

  return (
    <Container py={8}>
      <VStack gap={8} align="stretch">
        {/* Header */}
        <Box textAlign="center">
          <Heading size="xl" mb={2} color={marketNameColor}>
            Trading Activity
          </Heading>
          <Text color="gray.500" fontSize="lg">
            Your recent trades and market positions
          </Text>
        </Box>

        {/* Summary Stats */}
        <SimpleGrid columns={{ base: 2, md: 4 }} gap={6}>
          <Card.Root bg={cardBg} borderRadius="xl">
            <CardBody p={6} textAlign="center">
              <Stat.Root>
                <StatLabel color={outcomeColor} fontSize="sm">
                  Total Trades
                </StatLabel>
                <Stat.ValueText fontSize="3xl" color="blue.500">
                  {totalTrades}
                </Stat.ValueText>
                <Stat.HelpText>
                  <Icon mr={1}>
                    <Activity />
                  </Icon>
                  Active
                </Stat.HelpText>
              </Stat.Root>
            </CardBody>
          </Card.Root>

          <Card.Root bg={cardBg} borderRadius="xl">
            <CardBody p={6} textAlign="center">
              <Stat.Root>
                <Stat.Label color={outcomeColor} fontSize="sm">
                  Total Volume
                </Stat.Label>
                <Stat.ValueText fontSize="3xl" color="green.500">
                  {formatPrice(totalVolume.toString())}
                </Stat.ValueText>
                <StatHelpText>
                  <Icon mr={1}>
                    <DollarSign />
                  </Icon>
                  USD
                </StatHelpText>
              </Stat.Root>
            </CardBody>
          </Card.Root>

          <Card.Root bg={cardBg} borderRadius="xl">
            <CardBody p={6} textAlign="center">
              <Stat.Root>
                <StatLabel color={outcomeColor} fontSize="sm">
                  Buy Orders
                </StatLabel>
                <Stat.ValueText fontSize="3xl" color="green.500">
                  {buyTrades}
                </Stat.ValueText>
                <StatHelpText>
                  <Icon mr={1}>
                    <TrendingUp />
                  </Icon>
                  Long positions
                </StatHelpText>
              </Stat.Root>
            </CardBody>
          </Card.Root>

          <Card.Root bg={cardBg} borderRadius="xl">
            <CardBody p={6} textAlign="center">
              <Stat.Root>
                <StatLabel color={outcomeColor} fontSize="sm">
                  Sell Orders
                </StatLabel>
                <Stat.ValueText fontSize="3xl" color="red.500">
                  {sellTrades}
                </Stat.ValueText>
                <StatHelpText>
                  <Icon mr={1}>
                    <TrendingDown />
                  </Icon>
                  Short positions
                </StatHelpText>
              </Stat.Root>
            </CardBody>
          </Card.Root>
        </SimpleGrid>

        {/* Trades List */}
        <Card.Root bg={cardBg} borderRadius="xl" overflow="hidden">
          <CardHeader bg={useColorModeValue("gray.50", "gray.700")} py={4}>
            <HStack justify="space-between">
              <HStack>
                <Icon color="blue.500">
                  <BarChart3 />
                </Icon>
                <Heading size="md">Recent Trades</Heading>
              </HStack>
              <Badge
                colorScheme="blue"
                variant="solid"
                px={3}
                py={1}
                borderRadius="full"
              >
                {totalTrades} trades
              </Badge>
            </HStack>
          </CardHeader>

          <CardBody p={0}>
            <VStack gap={0} align="stretch">
              {tradesData.map((trade, index) => (
                <Box
                  key={index}
                  p={6}
                  borderBottom={index < tradesData.length - 1 ? "1px" : "none"}
                  borderColor={borderColor}
                  _hover={{ bg: hoverBg }}
                  transition="background-color 0.2s"
                >
                  <Flex
                    direction={{ base: "column", md: "row" }}
                    align={{ base: "start", md: "center" }}
                    justify="space-between"
                    gap={4}
                  >
                    {/* Market Info */}
                    <HStack gap={4} flex={1}>
                      <Avatar.Root
                        size="lg"
                        borderRadius="full"
                        border="2px"
                        borderColor={borderColor}
                      >
                        <Avatar.Image
                          src={trade.market_logo}
                          alt={trade.market_name}
                        />
                        <Avatar.Fallback>
                          {trade.market_name.charAt(0).toUpperCase()}
                        </Avatar.Fallback>
                      </Avatar.Root>

                      <VStack align="start" gap={1}>
                        <Heading size="sm" color={marketNameColor}>
                          {trade.market_name}
                        </Heading>
                        <HStack gap={2}>
                          <Badge
                            colorScheme={getStatusColor(trade.market_status)}
                            variant="solid"
                            size="sm"
                            borderRadius="full"
                          >
                            {trade.market_status.toUpperCase()}
                          </Badge>
                          <Badge
                            colorScheme={getOutcomeColor(trade.trade_outcome)}
                            variant="outline"
                            size="sm"
                            borderRadius="full"
                          >
                            {trade.trade_outcome.toUpperCase()}
                          </Badge>
                        </HStack>
                      </VStack>
                    </HStack>

                    {/* Trade Details */}
                    <HStack gap={6} align="center">
                      <VStack gap={1} align="center">
                        <Text
                          fontSize="xs"
                          color={typeColor}
                          fontWeight="medium"
                        >
                          TYPE
                        </Text>
                        <HStack>
                          <Icon
                            as={getTradeTypeIcon(trade.trade_type)}
                            color={`${getTradeTypeColor(trade.trade_type)}.500`}
                          />
                          <Badge
                            colorScheme={getTradeTypeColor(trade.trade_type)}
                            variant="solid"
                            textTransform="uppercase"
                            fontWeight="bold"
                          >
                            {trade.trade_type}
                          </Badge>
                        </HStack>
                      </VStack>

                      <Separator orientation="vertical" h="40px" />

                      <VStack gap={1} align="center">
                        <Text
                          fontSize="xs"
                          color={typeColor}
                          fontWeight="medium"
                        >
                          PRICE
                        </Text>
                        <Text fontWeight="bold" fontSize="lg">
                          {formatPrice(trade.trade_price)}
                        </Text>
                      </VStack>

                      <Separator orientation="vertical" h="40px" />

                      <VStack gap={1} align="center">
                        <Text
                          fontSize="xs"
                          color={typeColor}
                          fontWeight="medium"
                        >
                          QUANTITY
                        </Text>
                        <HStack>
                          <Icon boxSize={4} color="gray.400">
                            <ShoppingCart />
                          </Icon>
                          <Text fontWeight="bold" fontSize="lg">
                            {trade.trade_quantity}
                          </Text>
                        </HStack>
                      </VStack>

                      <Separator orientation="vertical" h="40px" />

                      <VStack gap={1} align="center">
                        <Text
                          fontSize="xs"
                          color={totalColor}
                          fontWeight="medium"
                        >
                          TOTAL
                        </Text>
                        <Text
                          fontWeight="bold"
                          fontSize="xl"
                          color={`${getTradeTypeColor(trade.trade_type)}.500`}
                        >
                          {calculateTotal(
                            trade.trade_price,
                            trade.trade_quantity,
                          )}
                        </Text>
                      </VStack>
                    </HStack>
                  </Flex>
                </Box>
              ))}

              <Pagination.Root
                pageSize={pageInfo.page_size}
                page={pageInfo.page}
                count={pageInfo.total_pages * pageInfo.page_size}
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
          </CardBody>
        </Card.Root>

        {/* Market Summary Card */}
        <SimpleGrid columns={{ base: 1, lg: 2 }} gap={6}>
          <Card.Root bg={cardBg} borderRadius="xl">
            <CardHeader>
              <Heading size="md">Market Overview</Heading>
            </CardHeader>
            <CardBody>
              <VStack gap={4} align="stretch">
                <HStack justify="space-between">
                  <Text color={statusColor}>Market Status</Text>
                  <Badge
                    colorScheme="green"
                    variant="solid"
                    px={3}
                    py={1}
                    borderRadius="full"
                  >
                    OPEN
                  </Badge>
                </HStack>
                <Separator />
                <HStack justify="space-between">
                  <Text color={statusColor}>Active Positions</Text>
                  <Text fontWeight="bold">{totalTrades}</Text>
                </HStack>
              </VStack>
            </CardBody>
          </Card.Root>

          <Card.Root bg={cardBg} borderRadius="xl">
            <CardHeader>
              <Heading size="md">Trading Summary</Heading>
            </CardHeader>
            <CardBody>
              <VStack gap={4} align="stretch">
                <HStack justify="space-between">
                  <HStack>
                    <Icon color="green.500">
                      <TrendingUp />
                    </Icon>
                    <Text color={statusColor}>Buy Orders</Text>
                  </HStack>
                  <Text fontWeight="bold" color="green.500">
                    {buyTrades}
                  </Text>
                </HStack>
                <Separator />
                <HStack justify="space-between">
                  <HStack>
                    <Icon color="red.500">
                      <TrendingDown />
                    </Icon>
                    <Text color={statusColor}>Sell Orders</Text>
                  </HStack>
                  <Text fontWeight="bold" color="red.500">
                    {sellTrades}
                  </Text>
                </HStack>
                <Separator />
                <HStack justify="space-between">
                  <HStack>
                    <Icon color="blue.500">
                      <DollarSign />
                    </Icon>
                    <Text color={statusColor}>Total Volume</Text>
                  </HStack>
                  <Text fontWeight="bold" color="blue.500">
                    {formatPrice(totalVolume.toString())}
                  </Text>
                </HStack>
              </VStack>
            </CardBody>
          </Card.Root>
        </SimpleGrid>
      </VStack>
    </Container>
  );
}
