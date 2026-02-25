"use client";

import EmptyStateCustom from "@/components/EmptyStateCustom";
import { useColorModeValue } from "@/components/ui/color-mode";
import { formatPriceString } from "@/utils";
import { UserGetters } from "@/utils/interactions/dataGetter";
import {
  Box,
  Container,
  Flex,
  Grid,
  Heading,
  Text,
  Avatar,
  Badge,
  Button,
  Card,
  CardBody,
  CardHeader,
  HStack,
  VStack,
  Icon,
  Code,
  SimpleGrid,
  Stack,
  Separator,
} from "@chakra-ui/react";
import { useQuery } from "@tanstack/react-query";
import {
  User,
  Mail,
  Calendar,
  TrendingUp,
  DollarSign,
  BarChart3,
  Activity,
  Wallet,
  Key,
  Clock,
} from "lucide-react";
import Link from "next/link";

function formatDate(dateString: string) {
  return new Date(dateString).toLocaleDateString("en-US", {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

export default function ProfilePage() {
  const { data } = useQuery({
    queryKey: ["userMetadata"],
    queryFn: UserGetters.getUserMetadata,
  });
  const cardBg = useColorModeValue("white", "gray.800");

  if (!data || !data.profile_insight) {
    return (
      <EmptyStateCustom
        title="No Profile Data"
        description="Unable to fetch your profile data at the moment. Please try again later."
      />
    );
  }
  const profileData = data.profile_insight;

  function handleDepositFunds() {
    // TODO
  }

  return (
    <Box minH="100vh" p={{ base: 4, md: 6, lg: 8 }}>
      <Container maxW="7xl" mx="auto">
        <VStack gap={6} align="stretch">
          {/* Header Section */}
          <Card.Root
            bgGradient="to-r"
            gradientFrom="blue.500"
            gradientTo="purple.600"
            color="white"
            borderRadius="xl"
            overflow="hidden"
          >
            <Card.Body p={{ base: 6, md: 8 }}>
              <Flex
                direction={{ base: "column", md: "row" }}
                align={{ base: "start", md: "center" }}
                gap={6}
              >
                <Avatar.Root
                  size={{ base: "xl", md: "2xl" }}
                  border="4px solid"
                  borderColor="whiteAlpha.300"
                >
                  <Avatar.Fallback name={profileData.name} />
                  <Avatar.Image src={profileData.avatar} />
                </Avatar.Root>

                <VStack align="start" flex={1} gap={2}>
                  <Heading size={{ base: "xl", md: "2xl" }} fontWeight="bold">
                    {profileData.name}
                  </Heading>
                  <HStack color="whiteAlpha.800">
                    <Icon>
                      <Mail />
                    </Icon>
                    <Text>{profileData.email}</Text>
                  </HStack>
                  <HStack color="whiteAlpha.800">
                    <Icon>
                      <Calendar />
                    </Icon>
                    <Text>
                      Member since {formatDate(profileData.created_at)}
                    </Text>
                  </HStack>
                </VStack>

                <VStack align={{ base: "start", md: "end" }} gap={2}>
                  <Text fontSize="sm" color="whiteAlpha.800">
                    Account Balance
                  </Text>
                  <Heading size={{ base: "xl", md: "2xl" }} fontWeight="bold">
                    {formatPriceString(profileData.balance)}
                  </Heading>
                  <Badge
                    colorScheme="whiteAlpha"
                    variant="solid"
                    px={3}
                    py={1}
                    borderRadius="full"
                  >
                    Active Trader
                  </Badge>
                </VStack>
              </Flex>
            </Card.Body>
          </Card.Root>

          {/* Stats Grid */}
          <SimpleGrid columns={{ base: 1, md: 2, lg: 4 }} gap={4}>
            <Card.Root bg={cardBg} borderRadius="lg">
              <Card.Body p={6}>
                <Flex justify="space-between" align="center">
                  <VStack align="start" gap={1}>
                    <Text fontSize="sm" fontWeight="medium" color="gray.500">
                      Total Trades
                    </Text>
                    <Heading size="lg">{profileData.total_trades}</Heading>
                  </VStack>
                  <Icon boxSize={8} color="green.500">
                    <TrendingUp />
                  </Icon>
                </Flex>
              </Card.Body>
            </Card.Root>

            <Card.Root bg={cardBg} borderRadius="lg">
              <Card.Body p={6}>
                <Flex justify="space-between" align="center">
                  <VStack align="start" gap={1}>
                    <Text fontSize="sm" fontWeight="medium" color="gray.500">
                      Total Volume
                    </Text>
                    <Heading size="lg">
                      {Number(profileData.total_volume).toFixed(3)}
                    </Heading>
                  </VStack>
                  <Icon boxSize={8} color="blue.500">
                    <BarChart3 />
                  </Icon>
                </Flex>
              </Card.Body>
            </Card.Root>

            <Card.Root bg={cardBg} borderRadius="lg">
              <Card.Body p={6}>
                <Flex justify="space-between" align="center">
                  <VStack align="start" gap={1}>
                    <Text fontSize="sm" fontWeight="medium" color="gray.500">
                      Avg Trade Price
                    </Text>
                    <Heading size="lg">
                      ${Number(profileData.avg_trade_price).toFixed(3)}
                    </Heading>
                  </VStack>
                  <Icon boxSize={8} color="yellow.500">
                    <DollarSign />
                  </Icon>
                </Flex>
              </Card.Body>
            </Card.Root>

            <Card.Root bg={cardBg} borderRadius="lg">
              <CardBody p={6}>
                <Flex justify="space-between" align="center">
                  <VStack align="start" gap={1}>
                    <Text fontSize="sm" fontWeight="medium" color="gray.500">
                      Fill Ratio
                    </Text>
                    <Heading size="lg">
                      {Number(profileData.avg_fill_ratio).toFixed(3)}
                    </Heading>
                  </VStack>
                  <Icon boxSize={8} color="purple.500">
                    <Activity />
                  </Icon>
                </Flex>
              </CardBody>
            </Card.Root>
          </SimpleGrid>

          {/* Detailed Information */}
          <Grid templateColumns={{ base: "1fr", lg: "1fr 1fr" }} gap={6}>
            {/* Trading Statistics */}
            <Card.Root bg={cardBg} borderRadius="lg">
              <Card.Header>
                <HStack>
                  <Icon>
                    <BarChart3 />
                  </Icon>
                  <Heading size="md">Trading Statistics</Heading>
                </HStack>
                <Text fontSize="sm" color="gray.500" mt={1}>
                  Your trading performance overview
                </Text>
              </Card.Header>
              <CardBody pt={0}>
                <VStack gap={4} align="stretch">
                  <Flex justify="space-between" align="center">
                    <Text fontSize="sm" fontWeight="medium">
                      Total Orders
                    </Text>
                    <Text fontWeight="bold">{profileData.total_orders}</Text>
                  </Flex>
                  <Separator />
                  <Flex justify="space-between" align="center">
                    <Text fontSize="sm" fontWeight="medium">
                      Markets Traded
                    </Text>
                    <Text fontWeight="bold">{profileData.markets_traded}</Text>
                  </Flex>
                  <Separator />
                  <Flex justify="space-between" align="center">
                    <Text fontSize="sm" fontWeight="medium">
                      Max Trade Quantity
                    </Text>
                    <Text fontWeight="bold">{profileData.max_trade_qty}</Text>
                  </Flex>
                  <Separator />
                  <Flex justify="space-between" align="center">
                    <Text fontSize="sm" fontWeight="medium">
                      Open Orders
                    </Text>
                    <Badge
                      colorScheme={
                        profileData.open_orders === 0 ? "gray" : "blue"
                      }
                      variant="solid"
                    >
                      {profileData.open_orders}
                    </Badge>
                  </Flex>
                  <Separator />
                  <Flex justify="space-between" align="center">
                    <Text fontSize="sm" fontWeight="medium">
                      Partial Orders
                    </Text>
                    <Badge
                      colorScheme={
                        profileData.partial_orders === 0 ? "gray" : "blue"
                      }
                      variant="solid"
                    >
                      {profileData.partial_orders}
                    </Badge>
                  </Flex>
                </VStack>
              </CardBody>
            </Card.Root>

            {/* Account Information */}
            <Card.Root bg={cardBg} borderRadius="lg">
              <CardHeader>
                <HStack>
                  <Icon>
                    <User />
                  </Icon>
                  <Heading size="md">Account Information</Heading>
                </HStack>
                <Text fontSize="sm" color="gray.500" mt={1}>
                  Your account details and activity
                </Text>
              </CardHeader>
              <CardBody pt={0}>
                <VStack gap={4} align="stretch">
                  <VStack align="start" gap={2}>
                    <HStack>
                      <Icon boxSize={4}>
                        <Key />
                      </Icon>
                      <Text fontSize="sm" fontWeight="medium">
                        Public Key
                      </Text>
                    </HStack>
                    <Box bg="gray.100" p={3} borderRadius="md" w="full">
                      <Code fontSize="xs" wordBreak="break-all">
                        {profileData.public_key}
                      </Code>
                    </Box>
                  </VStack>
                  <Separator />
                  <Flex justify="space-between" align="center">
                    <HStack>
                      <Icon boxSize={4}>
                        <Clock />
                      </Icon>
                      <Text fontSize="sm" fontWeight="medium">
                        First Trade
                      </Text>
                    </HStack>
                    <Text fontSize="sm">
                      {formatDate(profileData.first_trade_at)}
                    </Text>
                  </Flex>
                  <Separator />
                  <Flex justify="space-between" align="center">
                    <HStack>
                      <Icon boxSize={4}>
                        <Activity />
                      </Icon>
                      <Text fontSize="sm" fontWeight="medium">
                        Last Trade
                      </Text>
                    </HStack>
                    <Text fontSize="sm">
                      {formatDate(profileData.last_trade_at)}
                    </Text>
                  </Flex>
                  <Separator />
                  <Flex justify="space-between" align="center">
                    <HStack>
                      <Icon boxSize={4}>
                        <User />
                      </Icon>
                      <Text fontSize="sm" fontWeight="medium">
                        Last Login
                      </Text>
                    </HStack>
                    <Text fontSize="sm">
                      {formatDate(profileData.last_login)}
                    </Text>
                  </Flex>
                </VStack>
              </CardBody>
            </Card.Root>
          </Grid>

          {/* Action Buttons */}
          <Card.Root bg={cardBg} borderRadius="lg">
            <CardBody p={6}>
              <Stack
                direction={{ base: "column", sm: "row" }}
                gap={4}
                justify="center"
              >
                <Button
                  size="lg"
                  colorScheme="blue"
                  onClick={handleDepositFunds}
                >
                  <Icon>
                    <Wallet />
                  </Icon>
                  Deposit Funds
                </Button>
                <Link href="/profile/trades">
                  <Button size="lg" variant="outline" colorScheme="blue">
                    <Icon>
                      <TrendingUp />
                    </Icon>
                    View trades
                  </Button>
                </Link>
                <Link href="/profile/holdings">
                  <Button size="lg" variant="subtle" colorScheme="blue">
                    <Icon>
                      <BarChart3 />
                    </Icon>
                    View Holdings
                  </Button>
                </Link>
              </Stack>
            </CardBody>
          </Card.Root>
        </VStack>
      </Container>
    </Box>
  );
}
