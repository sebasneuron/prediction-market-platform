"use client";

import { Box, Tabs } from "@chakra-ui/react";
import OrderBook from "./_components/OrderBook";
import MyOrders from "./_components/MyOrders";
import TopMarketHolders from "./_components/TopMarketHolders";
import MarketTrades from "./_components/MarketTrades";

type Props = {
  marketId: string;
  yesPrice: number;
  noPrice: number;
};

const TabsClient = ({ marketId: id, noPrice, yesPrice }: Props) => {
  // TODO: add persistent state for the selected tab
  return (
    <div>
      <Box mt={10}>
        <Tabs.Root defaultValue="yes_book">
          <Tabs.List>
            <Tabs.Trigger value="yes_book">Trade yes</Tabs.Trigger>
            <Tabs.Trigger value="no_book">Trade no</Tabs.Trigger>
            <Tabs.Trigger value="my_orders">My orders</Tabs.Trigger>
            <Tabs.Trigger value="top_holders">Top holders</Tabs.Trigger>
            <Tabs.Trigger value="trades">Trades</Tabs.Trigger>
          </Tabs.List>
          <Tabs.Content value="yes_book">
            <OrderBook tradeType="yes" marketId={id} />
          </Tabs.Content>
          <Tabs.Content value="no_book">
            <OrderBook tradeType="no" marketId={id} />
          </Tabs.Content>
          <Tabs.Content value="my_orders">
            <MyOrders marketId={id} />
          </Tabs.Content>
          <Tabs.Content value="top_holders">
            <TopMarketHolders
              marketId={id}
              yesPrice={yesPrice}
              noPrice={noPrice}
            />
          </Tabs.Content>
          <Tabs.Content value="trades">
            <MarketTrades market_id={id} />
          </Tabs.Content>
        </Tabs.Root>
      </Box>
    </div>
  );
};

export default TabsClient;
