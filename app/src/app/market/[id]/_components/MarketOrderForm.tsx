"use client";

import { Box, Button, Flex, NumberInput, Text } from "@chakra-ui/react";
import { useState } from "react";
import { useMutation } from "@tanstack/react-query";

import { toaster } from "@/components/ui/toaster";
import useUserInfo from "@/hooks/useUserInfo";
import useRevalidation from "@/hooks/useRevalidate";
import { formatPriceString } from "@/utils";
import { MarketActions } from "@/utils/interactions/dataPosters";

type Props = {
  mode: "buy" | "sell";
  stockMode: "yes" | "no";
  market_id: string;
};

const MarketOrderForm = ({ mode, stockMode, market_id }: Props) => {
  const [amount, setAmount] = useState("");
  const { data: userInfo } = useUserInfo();
  const { mutateAsync, isPending } = useMutation({
    mutationFn: MarketActions.createMarketOrder,
  });
  const revalidate = useRevalidation();

  function handleSubmit() {
    if (amount === "") {
      toaster.error({
        title: "Amount is required",
      });
      return;
    }
    toaster.promise(
      mutateAsync({
        market_id,
        outcome: stockMode,
        price: Number(amount),
        side: mode,
      }),
      {
        loading: { title: "Creating order..." },
        success: () => {
          setAmount("");
          revalidate(["marketOrders", market_id]);
          revalidate(["userData"]);
          return { title: "Order created successfully!" };
        },
        error: (error: any) => {
          console.error("Error creating market order:", error);
          return {
            title: "Error",
            description: error.message || "Failed to create market order",
          };
        },
      },
    );
  }

  return (
    <Box>
      <Flex mt={4}>
        <Box width="full">
          <Text fontSize="lg" color="gray.600" fontWeight="semibold">
            Amount
          </Text>
          <Text fontSize="sm" color="gray.500" fontWeight="medium">
            Bal. {formatPriceString(userInfo?.balance || 0)}
          </Text>
        </Box>
        <NumberInput.Root
          formatOptions={{
            style: "currency",
            currency: "USD",
            currencyDisplay: "symbol",
            currencySign: "accounting",
          }}
        >
          <NumberInput.Input
            width="full"
            dir="rtl"
            outline="none"
            border="none"
            placeholder="$10"
            fontSize="4xl"
            fontWeight="extrabold"
            value={amount}
            onChange={(e) => setAmount(e.target.value)}
          />
        </NumberInput.Root>
      </Flex>
      {/* pre defined amount setter */}
      <Flex mt={3} justifyContent="end" alignItems="center">
        <Flex gap={2} alignItems="center">
          {PREDEFINED_AMOUNTS.map((amount) => (
            <Button
              key={amount}
              variant="outline"
              fontSize="xs"
              rounded="full"
              bg="transparent"
              border="1px solid"
              borderColor="gray.300"
              paddingX={5}
              size="xs"
              onClick={() =>
                setAmount((prev) => (Number(prev) + Number(amount)).toString())
              }
            >
              ${amount}
            </Button>
          ))}
        </Flex>
      </Flex>

      <Button
        width="full"
        mt={4}
        bg="blue.600/90"
        _hover={{ bg: "blue.600" }}
        onClick={handleSubmit}
        loading={isPending}
      >
        {mode === "buy" ? "Buy" : "Sell"} Now
      </Button>
    </Box>
  );
};

export default MarketOrderForm;
const PREDEFINED_AMOUNTS = [1, 20, 100];
