import {
  Box,
  Button,
  Flex,
  IconButton,
  NumberInput,
  Separator,
  Text,
} from "@chakra-ui/react";
import { Minus, Plus } from "lucide-react";
import React from "react";
import { useForm } from "react-hook-form";
import { useMutation } from "@tanstack/react-query";

import { toaster } from "@/components/ui/toaster";
import useRevalidation from "@/hooks/useRevalidate";
import useUserInfo from "@/hooks/useUserInfo";
import { MarketActions } from "@/utils/interactions/dataPosters";

type Props = {
  mode: "buy" | "sell";
  stockMode: "yes" | "no";
  market_id: string;
};

type FormState = {
  limitPrice: string;
  shares: string;
};

const LimitOrderForm = ({ mode, stockMode, market_id }: Props) => {
  const {
    handleSubmit,
    register,
    getValues,
    setValue,
    formState: { errors },
  } = useForm<FormState>();
  const { mutateAsync, isPending } = useMutation({
    mutationFn: MarketActions.createLimitOrder,
  });
  const { data: userInfo } = useUserInfo();
  const revalidate = useRevalidation();

  function onSubmit(data: FormState) {
    if (!userInfo) {
      toaster.info({
        title: "Please login to create an order",
        description: "You need to be logged in to place an order.",
      });
      return;
    }
    if (Number(data.limitPrice) < 0 || Number(data.limitPrice) > 100) {
      toaster.error({
        title: "Invalid price",
        description: "Price must be between $0 and $100.",
      });
      return;
    }
    toaster.promise(
      mutateAsync({
        market_id,
        outcome_side: stockMode,
        side: mode,
        price: Number(data.limitPrice),
        quantity: Number(data.shares),
      }),
      {
        loading: { title: "Creating order..." },
        success: () => {
          revalidate(["marketOrders", market_id]);
          revalidate(["userData"]);
          return { title: "Order created successfully!" };
        },
        error: (e: any) => {
          return {
            title: "Failed to create order",
            description: e?.message || "An error occurred",
          };
        },
      },
    );
  }
  return (
    <Box>
      <form onSubmit={handleSubmit(onSubmit)}>
        <Flex mt={4} alignItems="center" justifyContent={"space-between"}>
          <Text fontWeight="bold" fontSize="xl">
            Limit price
          </Text>
          <NumberInput.Root maxW="150px" step={0.01}>
            <Flex
              border="1px solid"
              rounded="md"
              borderColor={errors.limitPrice ? "red.400" : "gray.400"}
              alignItems="center"
              padding="0"
            >
              <NumberInput.DecrementTrigger asChild>
                <IconButton variant="ghost" size="xs" roundedRight="md">
                  <Minus />
                </IconButton>
              </NumberInput.DecrementTrigger>{" "}
              <NumberInput.Input
                placeholder="$2.00"
                width="full"
                textAlign="center"
                border="none"
                outline="none"
                fontWeight="semibold"
                color={errors.limitPrice ? "red.500" : "black"}
                {...register("limitPrice", {
                  required: "Limit price is required",
                  pattern: {
                    value: /^\d*\.?\d+$/,
                    message: "Invalid price format",
                  },
                  min: {
                    value: 1,
                    message: "Price must be at least 1",
                  },
                  max: {
                    value: 100,
                    message: "Price must be at most 100",
                  },
                })}
              />
              <NumberInput.IncrementTrigger asChild>
                <IconButton variant="ghost" size="xs">
                  <Plus />
                </IconButton>
              </NumberInput.IncrementTrigger>
            </Flex>
          </NumberInput.Root>
        </Flex>
        <Separator my={4} />
        <Box>
          <Flex alignItems="center" justifyContent="space-between">
            <Text fontWeight="bold" fontSize="xl">
              Shares
            </Text>
            <NumberInput.Root
              maxW="150px"
              step={10}
              border={"1px solid"}
              rounded="md"
              borderColor={errors.shares ? "red.400" : "gray.400"}
            >
              <NumberInput.Input
                placeholder="20"
                dir="rtl"
                border="none"
                outline="none"
                fontWeight="semibold"
                color={errors.shares ? "red.500" : "black"}
                {...register("shares", {
                  required: "Shares are required",
                  pattern: {
                    value: /^\d+$/,
                    message: "Invalid shares format",
                  },
                })}
              />
            </NumberInput.Root>
          </Flex>
          <Flex mt={3} justifyContent="end" gap={2} alignItems="center">
            {PREDEFINED_STOCKS.map((amount) => (
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
                  setValue(
                    "shares",
                    (Number(getValues("shares") || 0) + amount).toString(),
                  )
                }
              >
                +{amount}
              </Button>
            ))}
          </Flex>
        </Box>

        <Button
          type="submit"
          width="full"
          mt={4}
          bg="blue.600/90"
          _hover={{ bg: "blue.600" }}
          loading={isPending}
        >
          {mode === "buy" ? "Buy" : "Sell"} Now
        </Button>
      </form>
    </Box>
  );
};

export default LimitOrderForm;

const PREDEFINED_STOCKS = [1, 20, 100];
