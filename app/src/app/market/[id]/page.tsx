import { Avatar, Badge, Box, Container, Flex, Text } from "@chakra-ui/react";
import { Bookmark, Clock5, Link } from "lucide-react";

import EmptyStateCustom from "@/components/EmptyStateCustom";
import { MarketGetters } from "@/utils/interactions/dataGetter";

import PriceChart from "./_components/PriceChart";
import PurchaseNowActionBar from "./_components/PurchaseNowActionBar";
import VolumeInfoCard from "./_components/VolumeInfoCard";
import HoldingsInfoClient from "./_components/HoldingsInfoClient";
import TabsClient from "./TabsClient";

import { MarketStatus } from "@/generated/grpc_service_types/markets";

type Props = {
  params: Promise<{
    id: string;
  }>;
};

const MarketPage = async ({ params }: Props) => {
  const id = (await params).id;

  const marketWithVolume = await MarketGetters.getMarketById(id);

  if (
    !marketWithVolume ||
    !marketWithVolume.market ||
    !marketWithVolume.volumeInfo ||
    !marketWithVolume.marketPrice
  ) {
    return (
      <EmptyStateCustom
        title="Market not found"
        description="Please cross check the url as the market which you are looking is not found"
      />
    );
  }
  const market = marketWithVolume.market;
  const volumeInfo = marketWithVolume.volumeInfo;
  const marketPrice = marketWithVolume.marketPrice;

  return (
    <Container my={10}>
      <Box mb={20}>
        {/* avatar and title flex */}
        <Flex
          alignItems="center"
          justifyContent="space-between"
          mb={6}
          flexWrap="wrap"
        >
          <Flex alignItems="center" gap={3}>
            <Avatar.Root size="2xl" shape="rounded">
              <Avatar.Image src={market.logo} />
              <Avatar.Fallback name={market.name} />
            </Avatar.Root>
            <Text fontSize="2xl" fontWeight="semibold">
              {market.name}
            </Text>
            {market.status !== MarketStatus.OPEN && (
              <Badge variant="outline" rounded="lg" colorScheme="blue">
                {MarketStatus[market.status].toLowerCase()}
              </Badge>
            )}
          </Flex>

          <HoldingsInfoClient marketId={id} />
        </Flex>
        {/* volume and links */}
        <Flex mt={4} justifyContent="space-between" width="full">
          <Flex alignItems="center" gap={2}>
            <VolumeInfoCard volumeInfo={volumeInfo} />
            <Flex color="gray.600" fontSize="sm" alignItems={"center"} gap={1}>
              <Text>
                {new Date(market.marketExpiry).toLocaleDateString("en-US", {
                  year: "numeric",
                  month: "long",
                  day: "numeric",
                })}
              </Text>
              <Clock5 size={14} />
            </Flex>
          </Flex>
          <Flex alignItems="center" gap={3} color="gray.800">
            <Link size={16} />
            <Bookmark size={16} />
          </Flex>
        </Flex>

        {/* charts */}
        <PriceChart market_id={id} />

        {/*  action bar for purchasing now  */}
        {market.status === MarketStatus.OPEN && (
          <PurchaseNowActionBar
            market_id={id}
            marketPrice={{
              latestNoPrice: marketPrice.latestNoPrice,
              latestYesPrice: marketPrice.latestYesPrice,
              marketId: market.id,
            }}
          />
        )}

        {/* TABS */}
        <TabsClient
          marketId={id}
          yesPrice={marketPrice.latestYesPrice}
          noPrice={marketPrice.latestNoPrice}
        />
      </Box>
    </Container>
  );
};

export default MarketPage;
