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
  Icon,
  Stat,
  StatLabel,
  StatHelpText,
  Separator,
  Container,
  Progress,
  Button,
  Grid,
  GridItem,
  Pagination,
  ButtonGroup,
  IconButton,
} from "@chakra-ui/react";
import { useQuery } from "@tanstack/react-query";
import {
  TrendingUp,
  TrendingDown,
  Activity,
  BarChart3,
  PieChart,
  Calendar,
  Clock,
  Target,
  Eye,
  LucideChevronLeft,
  LucideChevronRight,
} from "lucide-react";
import Link from "next/link";
import { useState } from "react";

function formatDate(dateString: string) {
  return new Date(dateString).toLocaleDateString("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
  });
}

function formatDateTime(dateString: string) {
  return new Date(dateString).toLocaleDateString("en-US", {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function getTimeRemaining(expiryDate: string) {
  const now = new Date();
  const expiry = new Date(expiryDate);
  const diffTime = expiry.getTime() - now.getTime();
  const diffDays = Math.ceil(diffTime / (1000 * 60 * 60 * 24));

  if (diffDays < 0) return { text: "Expired", color: "red" };
  if (diffDays === 0) return { text: "Today", color: "orange" };
  if (diffDays <= 7) return { text: `${diffDays} days`, color: "orange" };
  if (diffDays <= 30) return { text: `${diffDays} days`, color: "yellow" };
  return { text: `${diffDays} days`, color: "green" };
}

function getOutcomeColor(outcome: string) {
  return outcome.toLowerCase() === "yes" ? "green" : "red";
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

function estimateValue(shares: string, outcome: string) {
  // Mock calculation - in real app, you'd use actual market prices
  const shareCount = parseInt(shares);
  const basePrice = outcome.toLowerCase() === "yes" ? 0.65 : 0.35;
  return (shareCount * basePrice).toFixed(2);
}

export default function HoldingsPage() {
  const [page, setPage] = useState(1);
  const { data } = useQuery({
    queryKey: ["HoldingsData"],
    queryFn: () => UserGetters.getUserHoldings(page, 10),
  });
  const cardBg = useColorModeValue("white", "gray.800");
  const borderColor = useColorModeValue("gray.200", "gray.600");
  const textColor = useColorModeValue("gray.800", "white");
  const mutedColor = useColorModeValue("gray.500", "gray.400");
  const positionDetailsBg = useColorModeValue("gray.50", "gray.700");
  const positionDetailsColorScheme = (outcome: string) =>
    outcome.toLowerCase() === "yes" ? "green" : "red";

  if (!data || !data.data || !data.data.holdings || !data.data.page_info) {
    return (
      <EmptyStateCustom
        title="No Holdings Found"
        description="You currently have no holdings in any markets."
      />
    );
  }
  const holdingsData = data.data.holdings;
  const pageInfo = data.data.page_info;

  // Calculate summary stats
  const totalHoldings = holdingsData.length;
  const totalShares = holdingsData.reduce(
    (sum, holding) => sum + parseInt(holding.shares),
    0,
  );
  const estimatedValue = holdingsData.reduce(
    (sum, holding) =>
      sum + parseFloat(estimateValue(holding.shares, holding.outcome)),
    0,
  );
  const yesPositions = holdingsData.filter(
    (h) => h.outcome.toLowerCase() === "yes",
  ).length;
  const noPositions = holdingsData.filter(
    (h) => h.outcome.toLowerCase() === "no",
  ).length;

  return (
    <Container maxW="7xl" py={8}>
      <VStack gap={8} align="stretch">
        {/* Header */}
        <Box textAlign="center">
          <Heading size="2xl" mb={2} color={textColor}>
            My Holdings
          </Heading>
          <Text color={mutedColor} fontSize="lg">
            Your prediction market portfolio and positions
          </Text>
        </Box>

        {/* Portfolio Summary */}
        <Card.Root bg={cardBg} borderRadius="2xl" overflow="hidden">
          <CardHeader
            bg="linear-gradient(135deg, #667eea 0%, #764ba2 100%)"
            color="white"
            py={8}
          >
            <VStack gap={4}>
              <Heading size="lg">Portfolio Overview</Heading>
              <HStack gap={8} justify="center">
                <VStack>
                  <Text fontSize="3xl" fontWeight="bold">
                    ${estimatedValue.toFixed(2)}
                  </Text>
                  <Text fontSize="sm" opacity={0.8}>
                    Estimated Value
                  </Text>
                </VStack>
                <Separator orientation="vertical" h="60px" />
                <VStack>
                  <Text fontSize="3xl" fontWeight="bold">
                    {totalShares.toLocaleString()}
                  </Text>
                  <Text fontSize="sm" opacity={0.8}>
                    Total Shares
                  </Text>
                </VStack>
                <Separator orientation="vertical" h="60px" />
                <VStack>
                  <Text fontSize="3xl" fontWeight="bold">
                    {totalHoldings}
                  </Text>
                  <Text fontSize="sm" opacity={0.8}>
                    Markets
                  </Text>
                </VStack>
              </HStack>
            </VStack>
          </CardHeader>
        </Card.Root>

        {/* Quick Stats */}
        <SimpleGrid columns={{ base: 2, md: 4 }} gap={6}>
          <Card.Root bg={cardBg} borderRadius="xl">
            <CardBody p={6} textAlign="center">
              <Stat.Root>
                <StatLabel color={mutedColor} fontSize="sm">
                  Active Positions
                </StatLabel>
                <Stat.ValueText fontSize="2xl" color="blue.500">
                  {totalHoldings}
                </Stat.ValueText>
                <StatHelpText>
                  <Icon mr={1}>
                    <Activity />
                  </Icon>
                  All markets
                </StatHelpText>
              </Stat.Root>
            </CardBody>
          </Card.Root>

          <Card.Root bg={cardBg} borderRadius="xl">
            <CardBody p={6} textAlign="center">
              <Stat.Root>
                <StatLabel color={mutedColor} fontSize="sm">
                  YES Positions
                </StatLabel>
                <Stat.ValueText fontSize="2xl" color="green.500">
                  {yesPositions}
                </Stat.ValueText>
                <StatHelpText>
                  <Icon mr={1}>
                    <TrendingUp />
                  </Icon>
                  Bullish bets
                </StatHelpText>
              </Stat.Root>
            </CardBody>
          </Card.Root>

          <Card.Root bg={cardBg} borderRadius="xl">
            <CardBody p={6} textAlign="center">
              <Stat.Root>
                <StatLabel color={mutedColor} fontSize="sm">
                  NO Positions
                </StatLabel>
                <Stat.ValueText fontSize="2xl" color="red.500">
                  {noPositions}
                </Stat.ValueText>
                <StatHelpText>
                  <Icon mr={1}>
                    <TrendingDown />
                  </Icon>
                  Bearish bets
                </StatHelpText>
              </Stat.Root>
            </CardBody>
          </Card.Root>

          <Card.Root bg={cardBg} borderRadius="xl">
            <CardBody p={6} textAlign="center">
              <Stat.Root>
                <StatLabel color={mutedColor} fontSize="sm">
                  Avg Position
                </StatLabel>
                <Stat.ValueText fontSize="2xl" color="purple.500">
                  {Math.round(totalShares / totalHoldings)}
                </Stat.ValueText>
                <StatHelpText>
                  <Icon mr={1}>
                    <PieChart />
                  </Icon>
                  Shares
                </StatHelpText>
              </Stat.Root>
            </CardBody>
          </Card.Root>
        </SimpleGrid>

        {/* Holdings Grid */}
        <Card.Root bg={cardBg} borderRadius="xl" overflow="hidden">
          <CardHeader bg={useColorModeValue("gray.50", "gray.700")} py={4}>
            <HStack justify="space-between">
              <HStack>
                <Icon color="blue.500">
                  <BarChart3 />
                </Icon>
                <Heading size="md">Your Positions</Heading>
              </HStack>
              <Badge
                colorScheme="blue"
                variant="solid"
                px={3}
                py={1}
                borderRadius="full"
              >
                {totalHoldings} holdings
              </Badge>
            </HStack>
          </CardHeader>

          <CardBody p={0}>
            <Grid
              templateColumns={{ base: "1fr", lg: "repeat(2, 1fr)" }}
              gap={0}
            >
              {holdingsData.map((holding, index) => {
                const timeRemaining = getTimeRemaining(holding.market_expiry);
                const estimatedVal = estimateValue(
                  holding.shares,
                  holding.outcome,
                );

                return (
                  <GridItem
                    key={holding.market_id}
                    p={6}
                    borderRight={{ lg: index % 2 === 0 ? "1px" : "none" }}
                    borderBottom={
                      index < holdingsData.length - 1 ? "1px" : "none"
                    }
                    borderColor={borderColor}
                    transition="background-color 0.2s"
                  >
                    <VStack gap={4} align="stretch">
                      {/* Market Header */}
                      <HStack gap={4}>
                        <Avatar.Root
                          size="lg"
                          borderRadius="lg"
                          border="2px"
                          borderColor={borderColor}
                        >
                          <Avatar.Image
                            src={holding.market_logo}
                            alt={holding.market_name}
                          />
                          <Avatar.Fallback>
                            {holding.market_name.charAt(0).toUpperCase()}
                          </Avatar.Fallback>
                        </Avatar.Root>

                        <VStack align="start" gap={1} flex={1}>
                          <Heading size="sm" color={textColor}>
                            {holding.market_name}
                          </Heading>
                          <Text fontSize="sm" color={mutedColor}>
                            {holding.market_description}
                          </Text>
                          <HStack gap={2}>
                            <Badge
                              colorScheme={getStatusColor(
                                holding.market_status,
                              )}
                              variant="solid"
                              size="sm"
                              borderRadius="full"
                            >
                              {holding.market_status.toUpperCase()}
                            </Badge>
                            <Badge
                              colorScheme={timeRemaining.color}
                              variant="outline"
                              size="sm"
                              borderRadius="full"
                            >
                              {timeRemaining.text} left
                            </Badge>
                          </HStack>
                        </VStack>
                      </HStack>

                      {/* Position Details */}
                      <Box bg={positionDetailsBg} p={4} borderRadius="lg">
                        <Grid templateColumns="repeat(3, 1fr)" gap={4}>
                          <VStack gap={1}>
                            <Text
                              fontSize="xs"
                              color={mutedColor}
                              fontWeight="medium"
                            >
                              POSITION
                            </Text>
                            <Badge
                              colorScheme={positionDetailsColorScheme(
                                holding.outcome,
                              )}
                              variant="solid"
                              size="lg"
                              px={3}
                              py={1}
                              borderRadius="full"
                              textTransform="uppercase"
                              fontWeight="bold"
                            >
                              {holding.outcome}
                            </Badge>
                          </VStack>

                          <VStack gap={1}>
                            <Text
                              fontSize="xs"
                              color={mutedColor}
                              fontWeight="medium"
                            >
                              SHARES
                            </Text>
                            <Text fontWeight="bold" fontSize="lg">
                              {parseInt(holding.shares).toLocaleString()}
                            </Text>
                          </VStack>

                          <VStack gap={1}>
                            <Text
                              fontSize="xs"
                              color={mutedColor}
                              fontWeight="medium"
                            >
                              EST. VALUE
                            </Text>
                            <Text
                              fontWeight="bold"
                              fontSize="lg"
                              color={
                                positionDetailsColorScheme(holding.outcome) +
                                ".500"
                              }
                            >
                              ${estimatedVal}
                            </Text>
                          </VStack>
                        </Grid>
                      </Box>

                      {/* Market Timeline */}
                      <HStack
                        justify="space-between"
                        fontSize="xs"
                        color={mutedColor}
                      >
                        <HStack gap={1}>
                          <Icon>
                            <Calendar />
                          </Icon>
                          <Text>
                            Created {formatDate(holding.market_created_at)}
                          </Text>
                        </HStack>
                        <HStack gap={1}>
                          <Icon>
                            <Clock />
                          </Icon>
                          <Text>
                            Expires {formatDate(holding.market_expiry)}
                          </Text>
                        </HStack>
                      </HStack>

                      {/* Action Buttons */}
                      <HStack gap={2}>
                        <Link href={`/market/${holding.market_id}`}>
                          <Button
                            size="sm"
                            variant="outline"
                            colorScheme="blue"
                            flex={1}
                          >
                            <Icon>
                              <Eye />
                            </Icon>
                            View Market
                          </Button>
                        </Link>
                      </HStack>
                    </VStack>
                  </GridItem>
                );
              })}
            </Grid>
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
          </CardBody>
        </Card.Root>

        {/* Position Distribution */}
        <SimpleGrid columns={{ base: 1, lg: 2 }} gap={6}>
          <Card.Root bg={cardBg} borderRadius="xl">
            <CardHeader>
              <HStack>
                <Icon color="blue.500">
                  <BarChart3 />
                </Icon>
                <Heading size="md">Position Distribution</Heading>
              </HStack>
            </CardHeader>
            <CardBody>
              <VStack gap={4}>
                <HStack justify="space-between" w="full">
                  <HStack>
                    <Box w={3} h={3} bg="green.500" borderRadius="full" />
                    <Text>YES Positions</Text>
                  </HStack>
                  <Text fontWeight="bold">{yesPositions}</Text>
                </HStack>
                <Progress.Root
                  value={(yesPositions / totalHoldings) * 100}
                  colorScheme="green"
                  size="lg"
                  borderRadius="full"
                  w="full"
                >
                  <Progress.Track>
                    <Progress.Range />
                  </Progress.Track>
                </Progress.Root>

                <HStack justify="space-between" w="full">
                  <HStack>
                    <Box w={3} h={3} bg="red.500" borderRadius="full" />
                    <Text>NO Positions</Text>
                  </HStack>
                  <Text fontWeight="bold">{noPositions}</Text>
                </HStack>
                <Progress.Root
                  value={(noPositions / totalHoldings) * 100}
                  colorScheme="red"
                  size="lg"
                  borderRadius="full"
                  w="full"
                >
                  <Progress.Track>
                    <Progress.Range />
                  </Progress.Track>
                </Progress.Root>
              </VStack>
            </CardBody>
          </Card.Root>

          <Card.Root bg={cardBg} borderRadius="xl">
            <CardHeader>
              <HStack>
                <Icon color="purple.500">
                  <Activity />
                </Icon>
                <Heading size="md">Portfolio Health</Heading>
              </HStack>
            </CardHeader>
            <CardBody>
              <VStack gap={4} align="stretch">
                <HStack justify="space-between">
                  <Text color={mutedColor}>Diversification</Text>
                  <Badge colorScheme="green" variant="solid">
                    Good
                  </Badge>
                </HStack>
                <Separator />
                <HStack justify="space-between">
                  <Text color={mutedColor}>Risk Level</Text>
                  <Badge colorScheme="yellow" variant="solid">
                    Medium
                  </Badge>
                </HStack>
                <Separator />
                <HStack justify="space-between">
                  <Text color={mutedColor}>Active Markets</Text>
                  <Text fontWeight="bold">{totalHoldings}</Text>
                </HStack>
                <Separator />
                <HStack justify="space-between">
                  <Text color={mutedColor}>Estimated Return</Text>
                  <Text fontWeight="bold" color="green.500">
                    +12.5%
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
