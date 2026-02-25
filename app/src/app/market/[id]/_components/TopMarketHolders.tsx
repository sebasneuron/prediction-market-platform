"use client";

import { MarketGetters } from "@/utils/interactions/dataGetter";
import React from "react";

import {
  Box,
  Table,
  Text,
  Badge,
  Avatar,
  HStack,
  VStack,
  Progress,
  Flex,
  Icon,
  Card,
  Heading,
} from "@chakra-ui/react";

import {
  TrendingUp,
  TrendingDown,
  Users,
  Award,
  Activity,
  Usb,
} from "lucide-react";
import { useColorModeValue } from "@/components/ui/color-mode";
import { useQuery } from "@tanstack/react-query";

type Props = {
  marketId: string;
  yesPrice: number;
  noPrice: number;
};

const TopMarketHolders = ({ marketId, noPrice, yesPrice }: Props) => {
  const { data: topHolders } = useQuery({
    queryKey: ["topHolders", marketId],
    queryFn: () => MarketGetters.getTopTenHolders(marketId),
  });

  const bgColor = useColorModeValue("white", "gray.800");
  const borderColor = useColorModeValue("gray.200", "gray.600");
  const headerBg = useColorModeValue("gray.50", "gray.700");
  const hoverBg = useColorModeValue("gray.50", "gray.700");

  const formatCurrency = (value: number) => {
    return new Intl.NumberFormat("en-US", {
      style: "currency",
      currency: "USD",
      minimumFractionDigits: 2,
    }).format(value);
  };

  const formatShares = (value: number) => {
    return new Intl.NumberFormat("en-US", {
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    }).format(value);
  };

  const getYesPercentage = (yesShares: number, totalShares: number) => {
    return (yesShares / totalShares) * 100;
  };

  const getNoPercentage = (noShares: number, totalShares: number) => {
    return (noShares / totalShares) * 100;
  };

  const getUserDisplayName = (userId: string) => {
    return userId
      .split("_")
      .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
      .join(" ");
  };

  const getRankIcon = (index: number) => {
    if (index === 0) return { icon: Award, color: "yellow.500" };
    if (index === 1) return { icon: Award, color: "gray.400" };
    if (index === 2) return { icon: Award, color: "orange.500" };
    return { icon: Users, color: "blue.500" };
  };

  function calculateTotalSharesPrice(
    yesShares: number,
    noShares: number,
  ): number {
    const yesPriceUsd = yesShares * yesPrice;
    const noPriceUsd = noShares * noPrice;
    return yesPriceUsd + noPriceUsd;
  }

  return (
    <Box>
      <Card.Root borderRadius="lg" py={4} mb={6}>
        <Card.Header>
          <HStack justify="space-between" align="center">
            <VStack align="start" gap={1}>
              <Heading size="lg"> Top Holders</Heading>
              <Text color="gray.500" fontSize="sm">
                Top market holders ranked by total shares held
              </Text>
            </VStack>
            <Icon boxSize={6} color="blue.500">
              <Usb />
            </Icon>
          </HStack>
        </Card.Header>
      </Card.Root>
      <Box>
        {/* from here--- */}
        <VStack gap={6} align="stretch">
          {/* Main Table */}
          <Box
            bg={bgColor}
            borderRadius="xl"
            border="1px"
            borderColor={borderColor}
            overflow="hidden"
          >
            <Table.Root variant="outline" size="md">
              <Table.Header bg={headerBg}>
                <Table.Row>
                  <Table.ColumnHeader
                    py={4}
                    fontSize="sm"
                    fontWeight="bold"
                    color="gray.600"
                  >
                    Rank & Trader
                  </Table.ColumnHeader>
                  <Table.ColumnHeader
                    py={4}
                    fontSize="sm"
                    fontWeight="bold"
                    color="gray.600"
                    textAlign="right"
                  >
                    Total Holdings
                  </Table.ColumnHeader>
                  <Table.ColumnHeader
                    py={4}
                    fontSize="sm"
                    fontWeight="bold"
                    color="gray.600"
                    textAlign="right"
                  >
                    Yes Shares
                  </Table.ColumnHeader>
                  <Table.ColumnHeader
                    py={4}
                    fontSize="sm"
                    fontWeight="bold"
                    color="gray.600"
                    textAlign="right"
                  >
                    No Shares
                  </Table.ColumnHeader>
                  <Table.ColumnHeader
                    py={4}
                    fontSize="sm"
                    fontWeight="bold"
                    color="gray.600"
                    textAlign="right"
                  >
                    Position Split
                  </Table.ColumnHeader>
                </Table.Row>
              </Table.Header>
              <Table.Body>
                {(topHolders || []).map((holder, index) => {
                  const rankInfo = getRankIcon(index);
                  const yesPercentage = getYesPercentage(
                    holder.totalYesShares,
                    holder.totalShares,
                  );
                  const noPercentage = getNoPercentage(
                    holder.totalNoShares,
                    holder.totalShares,
                  );

                  return (
                    <Table.Row
                      key={holder.userId}
                      _hover={{ bg: hoverBg }}
                      transition="background-color 0.2s"
                    >
                      <Table.Cell py={4}>
                        <HStack gap={3}>
                          <Flex
                            w={8}
                            h={8}
                            align="center"
                            justify="center"
                            bg={`${rankInfo.color.split(".")[0]}.100`}
                            borderRadius="full"
                          >
                            <Text
                              fontSize="sm"
                              fontWeight="bold"
                              color={rankInfo.color}
                            >
                              #{index + 1}
                            </Text>
                          </Flex>
                          <Avatar.Root size="sm">
                            <Avatar.Fallback
                              name={getUserDisplayName(holder.userId)}
                            />
                            <Avatar.Image
                              src={holder.avatar || ""}
                              alt={getUserDisplayName(holder.userId)}
                            />
                          </Avatar.Root>
                          <VStack align="start" gap={0}>
                            <Text fontWeight="semibold" fontSize="sm">
                              {getUserDisplayName(holder.username)}
                            </Text>
                          </VStack>
                          {index < 3 && (
                            <Icon
                              as={rankInfo.icon}
                              w={4}
                              h={4}
                              color={rankInfo.color}
                            />
                          )}
                        </HStack>
                      </Table.Cell>

                      <Table.Cell py={4}>
                        <VStack align="end" gap={1}>
                          <Text fontWeight="bold" fontSize="lg">
                            {formatShares(holder.totalShares)}
                          </Text>
                          <Text fontSize="xs" color="gray.500">
                            {formatCurrency(
                              calculateTotalSharesPrice(
                                holder.totalYesShares,
                                holder.totalNoShares,
                              ),
                            )}{" "}
                            value
                          </Text>
                        </VStack>
                      </Table.Cell>

                      <Table.Cell py={4}>
                        <VStack align="end" gap={1}>
                          <HStack gap={2}>
                            <Icon
                              as={TrendingUp}
                              w={3}
                              h={3}
                              color="green.500"
                            />
                            <Text fontWeight="semibold" color="green.600">
                              {formatShares(holder.totalYesShares)}
                            </Text>
                          </HStack>
                          <Badge
                            colorScheme="green"
                            variant="subtle"
                            fontSize="xs"
                          >
                            {yesPercentage.toFixed(1)}%
                          </Badge>
                        </VStack>
                      </Table.Cell>

                      <Table.Cell py={4}>
                        <VStack align="end" gap={1}>
                          <HStack gap={2}>
                            <Icon
                              as={TrendingDown}
                              w={3}
                              h={3}
                              color="red.500"
                            />
                            <Text fontWeight="semibold" color="red.600">
                              {formatShares(holder.totalNoShares)}
                            </Text>
                          </HStack>
                          <Badge
                            colorScheme="red"
                            variant="subtle"
                            fontSize="xs"
                          >
                            {noPercentage.toFixed(1)}%
                          </Badge>
                        </VStack>
                      </Table.Cell>

                      <Table.Cell py={4}>
                        <VStack gap={2} align="stretch" minW="120px">
                          <HStack justify="space-between">
                            <Text
                              fontSize="xs"
                              color="green.600"
                              fontWeight="medium"
                            >
                              Yes
                            </Text>
                            <Text
                              fontSize="xs"
                              color="red.600"
                              fontWeight="medium"
                            >
                              No
                            </Text>
                          </HStack>
                          <Progress.Root
                            value={yesPercentage}
                            max={100}
                            size="sm"
                            borderRadius="full"
                          >
                            <Progress.Track>
                              <Progress.Range />
                            </Progress.Track>
                          </Progress.Root>
                          <HStack
                            justify="space-between"
                            fontSize="xs"
                            color="gray.500"
                          >
                            <Text>{yesPercentage.toFixed(1)}%</Text>
                            <Text>{noPercentage.toFixed(1)}%</Text>
                          </HStack>
                        </VStack>
                      </Table.Cell>
                    </Table.Row>
                  );
                })}
              </Table.Body>
            </Table.Root>
          </Box>
        </VStack>
      </Box>
    </Box>
  );
};

export default TopMarketHolders;
