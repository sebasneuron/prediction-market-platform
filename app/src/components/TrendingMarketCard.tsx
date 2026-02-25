"use client";

import {
  Box,
  Card,
  CardBody,
  CardFooter,
  Button,
  Avatar,
  Heading,
  Text,
  HStack,
  VStack,
  Badge,
  Flex,
  Icon,
  Stat,
  StatLabel,
  StatHelpText,
  Progress,
  Separator,
} from "@chakra-ui/react";
import {
  TrendingUp,
  Clock,
  DollarSign,
  Activity,
  Calendar,
} from "lucide-react";
import Link from "next/link";
import { useColorModeValue } from "./ui/color-mode";
import { Market, MarketStatus } from "@/generated/grpc_service_types/markets";

interface Props {
  market: Market;
}

function formatDate(dateString: string) {
  return new Date(dateString).toLocaleDateString("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
  });
}

function getStatusColor(status: MarketStatus) {
  switch (status) {
    case MarketStatus.OPEN:
      return "green";
    case MarketStatus.CLOSED:
      return "orange";
    case MarketStatus.SETTLED:
      return "blue";
    default:
      return "gray";
  }
}

function getTimeRemaining(expiryDate: string) {
  const now = new Date();
  const expiry = new Date(expiryDate);
  const diffTime = expiry.getTime() - now.getTime();
  const diffDays = Math.ceil(diffTime / (1000 * 60 * 60 * 24));

  if (diffDays < 0) return "Expired";
  if (diffDays === 0) return "Today";
  if (diffDays === 1) return "1 day";
  return `${diffDays} days`;
}

const TrendingMarketCard = ({ market }: Props) => {
  const cardBg = useColorModeValue("white", "gray.800");
  const borderColor = useColorModeValue("gray.200", "gray.600");

  const yesPercentage = (market.yesPrice * 100).toFixed(0);
  const noPercentage = (market.noPrice * 100).toFixed(0);
  const timeRemaining = getTimeRemaining(market.marketExpiry);

  return (
    <Link href={`/market/${market.id}`} style={{ textDecoration: "none" }}>
      <Card.Root
        width="400px"
        h="320px"
        bg={cardBg}
        borderColor={borderColor}
        borderWidth="1px"
        borderRadius="xl"
        cursor="pointer"
        transition="all 0.2s"
        overflow="hidden"
      >
        <Card.Body p={5}>
          <VStack gap={4} align="stretch" h="full">
            {/* Header with Avatar and Status */}
            <Flex justify="space-between" align="start">
              <HStack gap={3}>
                <Avatar.Root size="md" borderRadius="full">
                  <Avatar.Image
                    boxSize="40px"
                    objectFit="cover"
                    src={market.logo}
                    alt={market.name}
                  />
                  <Avatar.Fallback>
                    {market.name.charAt(0).toUpperCase()}
                  </Avatar.Fallback>
                </Avatar.Root>

                <VStack align="start" gap={0} flex={1}>
                  <HStack gap={1} color="gray.500">
                    <Icon boxSize={3}>
                      <Clock />
                    </Icon>
                    <Text fontSize="xs">{timeRemaining} left</Text>
                  </HStack>
                </VStack>
              </HStack>
              <VStack align="end" gap={0}>
                <Badge
                  colorScheme={getStatusColor(market.status)}
                  variant="solid"
                  size="sm"
                  borderRadius="full"
                >
                  {MarketStatus[market.status]}
                </Badge>
              </VStack>
            </Flex>

            {/* Market Title */}
            <Box>
              <Heading
                size="sm"
                lineHeight="1.3"
                lineClamp={2}
                color={useColorModeValue("gray.800", "white")}
              >
                {market.name}
              </Heading>
            </Box>

            {/* Price Display */}
            <VStack gap={2} align="stretch">
              <HStack justify="space-between">
                <Text fontSize="sm" fontWeight="medium" color="gray.600">
                  Market Odds
                </Text>
                <HStack gap={1}>
                  <Icon boxSize={3} color="blue.500">
                    <TrendingUp />
                  </Icon>
                  <Text fontSize="xs" color="gray.500">
                    Live
                  </Text>
                </HStack>
              </HStack>

              <HStack gap={3}>
                <Stat.Root flex={1}>
                  <Stat.Label fontSize="xs" color="green.500">
                    YES
                  </Stat.Label>
                  <Stat.HelpText fontSize="lg" color="green.500">
                    {yesPercentage}$
                  </Stat.HelpText>
                </Stat.Root>
                <Separator orientation="vertical" h="40px" />
                <Stat.Root flex={1}>
                  <Stat.Label fontSize="xs" color="red.500">
                    NO
                  </Stat.Label>
                  <Stat.HelpText fontSize="lg" color="red.500">
                    {noPercentage}$
                  </Stat.HelpText>
                </Stat.Root>
              </HStack>

              {/* Progress Bar */}
              <Box>
                <Progress.Root
                  value={market.yesPrice * 100}
                  size="sm"
                  colorScheme="green"
                  bg="red.100"
                  borderRadius="full"
                >
                  <Progress.Track>
                    <Progress.Range />
                  </Progress.Track>
                </Progress.Root>

                <HStack justify="space-between" mt={1}>
                  <Text fontSize="xs" color="green.500" fontWeight="medium">
                    {yesPercentage}%
                  </Text>
                  <Text fontSize="xs" color="red.500" fontWeight="medium">
                    {noPercentage}%
                  </Text>
                </HStack>
              </Box>
            </VStack>

            {/* Expiry Date */}
            <HStack gap={1} color="gray.500" fontSize="xs">
              <Icon boxSize={3}>
                <Calendar />
              </Icon>
              <Text>Expires {formatDate(market.marketExpiry)}</Text>
            </HStack>
          </VStack>
        </Card.Body>

        <CardFooter pt={0} pb={4} px={5}>
          <HStack gap={2} w="full">
            <Button
              flex={1}
              variant="outline"
              colorScheme="green"
              size="sm"
              onClick={(e) => {
                e.preventDefault();
                e.stopPropagation();
                // Handle Buy Yes action
              }}
            >
              <Icon>
                <TrendingUp />
              </Icon>
              Buy Yes
            </Button>
            <Button
              flex={1}
              colorScheme="red"
              size="sm"
              onClick={(e) => {
                e.preventDefault();
                e.stopPropagation();
                // Handle Buy No action
              }}
            >
              <Icon>
                <Activity />
              </Icon>
              Buy No
            </Button>
          </HStack>
        </CardFooter>
      </Card.Root>
    </Link>
  );
};

export default TrendingMarketCard;
